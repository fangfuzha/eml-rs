//! Register-based bytecode execution for EML expressions.
//!
//! Compared with stack RPN, this representation supports:
//! - common subexpression elimination (CSE) via register reuse;
//! - constant folding into `LoadConst`.

use std::collections::hash_map::Entry;
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
        let mut regs = self.new_register_file()?;
        self.eval_complex_with_registers(vars, policy, &mut regs)
    }

    fn new_register_file(&self) -> Result<Vec<Complex64>, EmlError> {
        if self.register_count == 0 {
            return Err(EmlError::Unsupported("bytecode program has zero registers"));
        }
        Ok(vec![Complex64::new(0.0, 0.0); self.register_count])
    }

    fn eval_complex_with_registers(
        &self,
        vars: &[Complex64],
        policy: &EvalPolicy,
        regs: &mut [Complex64],
    ) -> Result<Complex64, EmlError> {
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

    fn eval_real_with_registers(
        &self,
        vars: &[f64],
        imag_tolerance: f64,
        policy: &EvalPolicy,
        regs: &mut [Complex64],
    ) -> Result<f64, EmlError> {
        for inst in &self.instructions {
            match inst {
                Instruction::LoadOne { dst } => {
                    regs[*dst] = Complex64::new(1.0, 0.0);
                }
                Instruction::LoadVar { dst, index } => {
                    let value = vars.get(*index).copied().ok_or(EmlError::MissingVariable {
                        index: *index,
                        arity: vars.len(),
                    })?;
                    regs[*dst] = Complex64::new(value, 0.0);
                }
                Instruction::LoadConst { dst, value } => {
                    regs[*dst] = *value;
                }
                Instruction::Eml { dst, lhs, rhs } => {
                    regs[*dst] = eml_complex_with_policy(regs[*lhs], regs[*rhs], policy)?;
                }
            }
        }

        let out = regs[self.output];
        if out.im.abs() > imag_tolerance {
            return Err(EmlError::NonRealOutput {
                imag: out.im,
                tolerance: imag_tolerance,
            });
        }
        Ok(out.re)
    }

    /// Executes bytecode over complex inputs with default policy.
    pub fn eval_complex(&self, vars: &[Complex64]) -> Result<Complex64, EmlError> {
        self.eval_complex_with_policy(vars, &EvalPolicy::default())
    }

    /// Executes bytecode over a complex batch while reusing one register file.
    pub fn eval_complex_batch_with_policy(
        &self,
        samples: &[Vec<Complex64>],
        policy: &EvalPolicy,
    ) -> Result<Vec<Complex64>, EmlError> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let mut regs = self.new_register_file()?;
        let mut out = Vec::with_capacity(samples.len());
        for vars in samples {
            out.push(self.eval_complex_with_registers(vars, policy, &mut regs)?);
        }
        Ok(out)
    }

    /// Executes bytecode over a complex batch with default policy.
    pub fn eval_complex_batch(
        &self,
        samples: &[Vec<Complex64>],
    ) -> Result<Vec<Complex64>, EmlError> {
        self.eval_complex_batch_with_policy(samples, &EvalPolicy::default())
    }

    /// Executes bytecode over real inputs with explicit policy.
    pub fn eval_real_with_policy(
        &self,
        vars: &[f64],
        imag_tolerance: f64,
        policy: &EvalPolicy,
    ) -> Result<f64, EmlError> {
        let mut regs = self.new_register_file()?;
        self.eval_real_with_registers(vars, imag_tolerance, policy, &mut regs)
    }

    /// Executes bytecode over real inputs with default policy.
    pub fn eval_real(&self, vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
        self.eval_real_with_policy(vars, imag_tolerance, &EvalPolicy::default())
    }

    /// Executes bytecode over a real batch while reusing one register file.
    pub fn eval_real_batch_with_policy(
        &self,
        samples: &[Vec<f64>],
        imag_tolerance: f64,
        policy: &EvalPolicy,
    ) -> Result<Vec<f64>, EmlError> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let mut regs = self.new_register_file()?;
        let mut out = Vec::with_capacity(samples.len());
        for vars in samples {
            out.push(self.eval_real_with_registers(vars, imag_tolerance, policy, &mut regs)?);
        }
        Ok(out)
    }

    /// Executes bytecode over a real batch with default policy.
    pub fn eval_real_batch(
        &self,
        samples: &[Vec<f64>],
        imag_tolerance: f64,
    ) -> Result<Vec<f64>, EmlError> {
        self.eval_real_batch_with_policy(samples, imag_tolerance, &EvalPolicy::default())
    }
}

/// Internal builder that performs CSE and constant folding.
struct Builder<'a> {
    instructions: Vec<Instruction>,
    next_reg: usize,
    next_key: usize,
    key_ids: HashMap<NodeKey, usize>,
    compiled: HashMap<usize, CompiledNode>,
    policy: &'a EvalPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeKey {
    One,
    Var(usize),
    Eml(usize, usize),
}

#[derive(Debug, Clone, Copy)]
struct CompiledNode {
    reg: usize,
    const_value: Option<Complex64>,
}

#[derive(Debug, Clone, Copy)]
struct CompileEntry {
    key_id: usize,
    reg: usize,
    const_value: Option<Complex64>,
}

impl<'a> Builder<'a> {
    fn new(policy: &'a EvalPolicy) -> Self {
        Self {
            instructions: Vec::new(),
            next_reg: 0,
            next_key: 0,
            key_ids: HashMap::new(),
            compiled: HashMap::new(),
            policy,
        }
    }

    fn alloc(&mut self) -> usize {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    fn intern_key(&mut self, key: NodeKey) -> usize {
        match self.key_ids.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.next_key;
                self.next_key += 1;
                entry.insert(id);
                id
            }
        }
    }

    fn cached_entry(&self, key_id: usize) -> Option<CompileEntry> {
        self.compiled.get(&key_id).map(|compiled| CompileEntry {
            key_id,
            reg: compiled.reg,
            const_value: compiled.const_value,
        })
    }

    fn compile_leaf(
        &mut self,
        key: NodeKey,
        const_value: Option<Complex64>,
        instruction: Instruction,
    ) -> CompileEntry {
        let key_id = self.intern_key(key);
        if let Some(entry) = self.cached_entry(key_id) {
            return entry;
        }

        let reg = self.alloc();
        let inst = match instruction {
            Instruction::LoadOne { .. } => Instruction::LoadOne { dst: reg },
            Instruction::LoadVar { index, .. } => Instruction::LoadVar { dst: reg, index },
            Instruction::LoadConst { value, .. } => Instruction::LoadConst { dst: reg, value },
            Instruction::Eml { .. } => {
                unreachable!("leaf compilation only accepts leaf instructions")
            }
        };
        self.instructions.push(inst);
        self.compiled
            .insert(key_id, CompiledNode { reg, const_value });
        CompileEntry {
            key_id,
            reg,
            const_value,
        }
    }

    fn compile_cse(&mut self, expr: &Expr) -> Result<usize, EmlError> {
        let CompileEntry { reg, .. } = self.compile_entry(expr)?;
        Ok(reg)
    }

    fn compile_entry(&mut self, expr: &Expr) -> Result<CompileEntry, EmlError> {
        match expr {
            Expr::One => Ok(self.compile_leaf(
                NodeKey::One,
                Some(Complex64::new(1.0, 0.0)),
                Instruction::LoadOne { dst: 0 },
            )),
            Expr::Var(index) => Ok(self.compile_leaf(
                NodeKey::Var(*index),
                None,
                Instruction::LoadVar {
                    dst: 0,
                    index: *index,
                },
            )),
            Expr::Eml(lhs, rhs) => {
                let lhs_entry = self.compile_entry(lhs)?;
                let rhs_entry = self.compile_entry(rhs)?;
                let key_id = self.intern_key(NodeKey::Eml(lhs_entry.key_id, rhs_entry.key_id));
                if let Some(entry) = self.cached_entry(key_id) {
                    return Ok(entry);
                }

                let const_value = match (lhs_entry.const_value, rhs_entry.const_value) {
                    (Some(l), Some(r)) => eml_complex_with_policy(l, r, self.policy).ok(),
                    _ => None,
                };
                let reg = self.alloc();
                if let Some(value) = const_value {
                    self.instructions
                        .push(Instruction::LoadConst { dst: reg, value });
                } else {
                    self.instructions.push(Instruction::Eml {
                        dst: reg,
                        lhs: lhs_entry.reg,
                        rhs: rhs_entry.reg,
                    });
                }
                self.compiled
                    .insert(key_id, CompiledNode { reg, const_value });
                Ok(CompileEntry {
                    key_id,
                    reg,
                    const_value,
                })
            }
        }
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

    #[test]
    fn complex_batch_eval_matches_scalar_bytecode() {
        let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::exp(Expr::var(1)));
        let prog = BytecodeProgram::from_expr(&expr).unwrap();
        let samples = vec![
            vec![Complex64::new(0.1, 0.0), Complex64::new(0.2, 0.0)],
            vec![Complex64::new(0.3, 0.0), Complex64::new(0.4, 0.0)],
            vec![Complex64::new(0.5, 0.0), Complex64::new(0.6, 0.0)],
        ];

        let scalar: Vec<_> = samples
            .iter()
            .map(|vars| prog.eval_complex(vars).unwrap())
            .collect();
        let batch = prog.eval_complex_batch(&samples).unwrap();

        assert_eq!(batch, scalar);
    }

    #[test]
    fn real_batch_eval_matches_scalar_bytecode() {
        let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::exp(Expr::var(1)));
        let prog = BytecodeProgram::from_expr(&expr).unwrap();
        let samples = vec![vec![0.1, 0.2], vec![0.3, 0.4], vec![0.5, 0.6]];

        let scalar: Vec<_> = samples
            .iter()
            .map(|vars| prog.eval_real(vars, 1e-12).unwrap())
            .collect();
        let batch = prog.eval_real_batch(&samples, 1e-12).unwrap();

        assert_eq!(batch, scalar);
    }
}
