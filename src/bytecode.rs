//! Register-based bytecode execution for EML expressions.
//!
//! Compared with stack RPN, this representation supports:
//! - common subexpression elimination (CSE) via register reuse;
//! - constant folding into `LoadConst`.

use std::collections::HashMap;

use num_complex::Complex64;

use crate::core::{eml_complex_with_policy, EvalPolicy};
use crate::ir::Expr;
use crate::EmlError;

/// Single bytecode instruction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    /// `r[dst] = 1`
    LoadOne { dst: usize },
    /// `r[dst] = vars[index]`
    LoadVar { dst: usize, index: usize },
    /// `r[dst] = value`
    LoadConst { dst: usize, value: Complex64 },
    /// `r[dst] = eml(r[lhs], r[rhs])`
    Eml { dst: usize, lhs: usize, rhs: usize },
}

/// Compiled register program.
#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeProgram {
    /// Linear instruction sequence.
    pub instructions: Vec<Instruction>,
    /// Output register index.
    pub output: usize,
    /// Register file size.
    pub register_count: usize,
}

impl BytecodeProgram {
    /// Compiles expression to bytecode with CSE and constant folding.
    pub fn from_expr(expr: &Expr) -> Result<Self, EmlError> {
        Self::from_expr_with_policy(expr, &EvalPolicy::default())
    }

    /// Compiles expression to bytecode with CSE and constant folding under policy.
    ///
    /// The policy is used while evaluating constant subtrees.
    pub fn from_expr_with_policy(expr: &Expr, policy: &EvalPolicy) -> Result<Self, EmlError> {
        let mut builder = Builder::new(policy);
        let output = builder.compile_cse(expr)?;
        Ok(Self {
            instructions: builder.instructions,
            output,
            register_count: builder.next_reg.max(output + 1),
        })
    }

    /// Compiles expression to a naive register program without CSE/folding.
    pub fn from_expr_naive(expr: &Expr) -> Self {
        let mut instructions = Vec::<Instruction>::new();
        let mut next_reg = 0usize;

        fn alloc(next_reg: &mut usize) -> usize {
            let r = *next_reg;
            *next_reg += 1;
            r
        }

        fn walk(expr: &Expr, next_reg: &mut usize, out: &mut Vec<Instruction>) -> usize {
            match expr {
                Expr::One => {
                    let dst = alloc(next_reg);
                    out.push(Instruction::LoadOne { dst });
                    dst
                }
                Expr::Var(index) => {
                    let dst = alloc(next_reg);
                    out.push(Instruction::LoadVar { dst, index: *index });
                    dst
                }
                Expr::Eml(lhs, rhs) => {
                    let l = walk(lhs, next_reg, out);
                    let r = walk(rhs, next_reg, out);
                    let dst = alloc(next_reg);
                    out.push(Instruction::Eml {
                        dst,
                        lhs: l,
                        rhs: r,
                    });
                    dst
                }
            }
        }

        let output = walk(expr, &mut next_reg, &mut instructions);
        Self {
            instructions,
            output,
            register_count: next_reg.max(output + 1),
        }
    }

    /// Executes bytecode over complex inputs with explicit policy.
    pub fn eval_complex_with_policy(
        &self,
        vars: &[Complex64],
        policy: &EvalPolicy,
    ) -> Result<Complex64, EmlError> {
        if self.register_count == 0 {
            return Err(EmlError::Unsupported("bytecode program has zero registers"));
        }

        let mut regs = vec![Complex64::new(0.0, 0.0); self.register_count];
        for inst in &self.instructions {
            match inst {
                Instruction::LoadOne { dst } => {
                    regs[*dst] = Complex64::new(1.0, 0.0);
                }
                Instruction::LoadVar { dst, index } => {
                    regs[*dst] = vars.get(*index).copied().ok_or(EmlError::MissingVariable {
                        index: *index,
                        arity: vars.len(),
                    })?;
                }
                Instruction::LoadConst { dst, value } => {
                    regs[*dst] = *value;
                }
                Instruction::Eml { dst, lhs, rhs } => {
                    regs[*dst] = eml_complex_with_policy(regs[*lhs], regs[*rhs], policy)?;
                }
            }
        }
        Ok(regs[self.output])
    }

    /// Executes bytecode over complex inputs with default policy.
    pub fn eval_complex(&self, vars: &[Complex64]) -> Result<Complex64, EmlError> {
        self.eval_complex_with_policy(vars, &EvalPolicy::default())
    }

    /// Executes bytecode over real inputs with explicit policy.
    pub fn eval_real_with_policy(
        &self,
        vars: &[f64],
        imag_tolerance: f64,
        policy: &EvalPolicy,
    ) -> Result<f64, EmlError> {
        let vars_complex: Vec<Complex64> = vars.iter().map(|v| Complex64::new(*v, 0.0)).collect();
        let out = self.eval_complex_with_policy(&vars_complex, policy)?;
        if out.im.abs() > imag_tolerance {
            return Err(EmlError::NonRealOutput {
                imag: out.im,
                tolerance: imag_tolerance,
            });
        }
        Ok(out.re)
    }

    /// Executes bytecode over real inputs with default policy.
    pub fn eval_real(&self, vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
        self.eval_real_with_policy(vars, imag_tolerance, &EvalPolicy::default())
    }
}

/// Internal builder that performs CSE and constant folding.
struct Builder<'a> {
    instructions: Vec<Instruction>,
    next_reg: usize,
    cse: HashMap<String, usize>,
    const_cache: HashMap<String, Option<Complex64>>,
    policy: &'a EvalPolicy,
}

impl<'a> Builder<'a> {
    fn new(policy: &'a EvalPolicy) -> Self {
        Self {
            instructions: Vec::new(),
            next_reg: 0,
            cse: HashMap::new(),
            const_cache: HashMap::new(),
            policy,
        }
    }

    fn alloc(&mut self) -> usize {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    fn compile_cse(&mut self, expr: &Expr) -> Result<usize, EmlError> {
        let key = expr.fingerprint();
        if let Some(reg) = self.cse.get(&key).copied() {
            return Ok(reg);
        }

        let reg = if let Some(value) = self.const_value(expr) {
            let dst = self.alloc();
            self.instructions
                .push(Instruction::LoadConst { dst, value });
            dst
        } else {
            match expr {
                Expr::One => {
                    let dst = self.alloc();
                    self.instructions.push(Instruction::LoadOne { dst });
                    dst
                }
                Expr::Var(index) => {
                    let dst = self.alloc();
                    self.instructions
                        .push(Instruction::LoadVar { dst, index: *index });
                    dst
                }
                Expr::Eml(lhs, rhs) => {
                    let l = self.compile_cse(lhs)?;
                    let r = self.compile_cse(rhs)?;
                    let dst = self.alloc();
                    self.instructions.push(Instruction::Eml {
                        dst,
                        lhs: l,
                        rhs: r,
                    });
                    dst
                }
            }
        };

        self.cse.insert(key, reg);
        Ok(reg)
    }

    fn const_value(&mut self, expr: &Expr) -> Option<Complex64> {
        let key = expr.fingerprint();
        if let Some(cached) = self.const_cache.get(&key) {
            return *cached;
        }

        let val = match expr {
            Expr::One => Some(Complex64::new(1.0, 0.0)),
            Expr::Var(_) => None,
            Expr::Eml(lhs, rhs) => {
                let l = self.const_value(lhs)?;
                let r = self.const_value(rhs)?;
                eml_complex_with_policy(l, r, self.policy).ok()
            }
        };
        self.const_cache.insert(key, val);
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ir::Expr;

    #[test]
    fn bytecode_matches_tree_eval() {
        let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::ln(Expr::var(1)));
        let prog = BytecodeProgram::from_expr(&expr).unwrap();

        let vars = vec![Complex64::new(0.4, -0.1), Complex64::new(1.3, 0.2)];
        let tree = expr.eval_complex(&vars).unwrap();
        let bytecode = prog.eval_complex(&vars).unwrap();
        assert!((tree - bytecode).norm() <= 1e-12);
    }

    #[test]
    fn cse_reduces_instruction_count_on_repeated_subtree() {
        let repeated = Expr::exp(Expr::var(0));
        let expr = Expr::eml(repeated.clone(), repeated);

        let naive = BytecodeProgram::from_expr_naive(&expr);
        let cse = BytecodeProgram::from_expr(&expr).unwrap();
        assert!(cse.instructions.len() < naive.instructions.len());
    }

    #[test]
    fn const_folding_emits_load_const() {
        let expr = Expr::ln(Expr::one());
        let prog = BytecodeProgram::from_expr(&expr).unwrap();
        assert!(prog
            .instructions
            .iter()
            .any(|inst| matches!(inst, Instruction::LoadConst { .. })));
    }
}
