//! EML expression IR, RPN conversion/evaluation, and structural statistics.
//!
//! The core expression grammar intentionally keeps only:
//! `One`, `Var(i)`, and `Eml(lhs, rhs)`.

use std::collections::{HashMap, HashSet};

use num_complex::Complex64;

use crate::core::{eml_complex_with_policy, EvalPolicy};
use crate::EmlError;

/// Expression tree for EML formulas.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Constant `1`.
    One,
    /// Variable by zero-based index.
    Var(usize),
    /// EML node `exp(lhs) - ln(rhs)`.
    Eml(Box<Expr>, Box<Expr>),
}

/// Reverse polish notation tokens for [`Expr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// Constant `1`.
    One,
    /// Variable by index.
    Var(usize),
    /// Apply EML to top-2 stack values.
    Eml,
}

/// Structural metrics for an expression tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprStats {
    /// Total number of nodes.
    pub nodes: usize,
    /// Maximum depth where a single node has depth 1.
    pub depth: usize,
    /// Number of EML interior nodes.
    pub eml_nodes: usize,
    /// Distinct variable indices referenced by the tree.
    pub distinct_vars: usize,
    /// Maximum referenced variable index + 1 (arity lower-bound).
    pub min_required_arity: usize,
    /// Number of unique subtree fingerprints.
    pub unique_subexpressions: usize,
    /// Number of duplicated subtree instances (sum of count-1).
    pub shared_subexpressions: usize,
}

impl Expr {
    /// Creates constant-one expression.
    pub fn one() -> Self {
        Self::One
    }

    /// Creates variable expression by index.
    pub fn var(index: usize) -> Self {
        Self::Var(index)
    }

    /// Creates an EML node.
    pub fn eml(lhs: Expr, rhs: Expr) -> Self {
        Self::Eml(Box::new(lhs), Box::new(rhs))
    }

    /// Builds `exp(arg)` as `eml(arg, 1)`.
    pub fn exp(arg: Expr) -> Self {
        Self::eml(arg, Self::one())
    }

    /// Builds `ln(arg)` using only EML primitives.
    ///
    /// Formula:
    /// `ln(x) = eml(1, eml(eml(1, x), 1))`.
    pub fn ln(arg: Expr) -> Self {
        Self::eml(
            Self::one(),
            Self::eml(Self::eml(Self::one(), arg), Self::one()),
        )
    }

    /// Evaluates expression using explicit policy.
    pub fn eval_complex_with_policy(
        &self,
        vars: &[Complex64],
        policy: &EvalPolicy,
    ) -> Result<Complex64, EmlError> {
        match self {
            Expr::One => Ok(Complex64::new(1.0, 0.0)),
            Expr::Var(index) => vars.get(*index).copied().ok_or(EmlError::MissingVariable {
                index: *index,
                arity: vars.len(),
            }),
            Expr::Eml(lhs, rhs) => {
                let l = lhs.eval_complex_with_policy(vars, policy)?;
                let r = rhs.eval_complex_with_policy(vars, policy)?;
                eml_complex_with_policy(l, r, policy)
            }
        }
    }

    /// Evaluates expression with default policy.
    pub fn eval_complex(&self, vars: &[Complex64]) -> Result<Complex64, EmlError> {
        self.eval_complex_with_policy(vars, &EvalPolicy::default())
    }

    /// Evaluates expression over real inputs with explicit policy.
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

    /// Evaluates expression over real inputs with default policy.
    pub fn eval_real(&self, vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
        self.eval_real_with_policy(vars, imag_tolerance, &EvalPolicy::default())
    }

    /// Converts expression into a fresh RPN token sequence.
    pub fn to_rpn_vec(&self) -> Vec<Token> {
        let mut out = Vec::new();
        self.to_rpn(&mut out);
        out
    }

    /// Computes structural statistics over the expression tree.
    pub fn stats(&self) -> ExprStats {
        let mut counts = HashMap::<String, usize>::new();
        let mut var_set = HashSet::<usize>::new();
        let mut nodes = 0usize;
        let mut eml_nodes = 0usize;
        let mut min_required_arity = 0usize;

        fn walk(
            expr: &Expr,
            depth: usize,
            nodes: &mut usize,
            eml_nodes: &mut usize,
            var_set: &mut HashSet<usize>,
            min_required_arity: &mut usize,
            counts: &mut HashMap<String, usize>,
            max_depth: &mut usize,
        ) -> String {
            *nodes += 1;
            *max_depth = (*max_depth).max(depth);
            let key = match expr {
                Expr::One => "1".to_string(),
                Expr::Var(index) => {
                    var_set.insert(*index);
                    *min_required_arity = (*min_required_arity).max(*index + 1);
                    format!("v{index}")
                }
                Expr::Eml(lhs, rhs) => {
                    *eml_nodes += 1;
                    let lk = walk(
                        lhs,
                        depth + 1,
                        nodes,
                        eml_nodes,
                        var_set,
                        min_required_arity,
                        counts,
                        max_depth,
                    );
                    let rk = walk(
                        rhs,
                        depth + 1,
                        nodes,
                        eml_nodes,
                        var_set,
                        min_required_arity,
                        counts,
                        max_depth,
                    );
                    format!("e({lk},{rk})")
                }
            };
            *counts.entry(key.clone()).or_insert(0) += 1;
            key
        }

        let mut depth = 0usize;
        walk(
            self,
            1,
            &mut nodes,
            &mut eml_nodes,
            &mut var_set,
            &mut min_required_arity,
            &mut counts,
            &mut depth,
        );

        let shared_subexpressions = counts
            .values()
            .filter(|count| **count > 1)
            .map(|count| count - 1)
            .sum();

        ExprStats {
            nodes,
            depth,
            eml_nodes,
            distinct_vars: var_set.len(),
            min_required_arity,
            unique_subexpressions: counts.len(),
            shared_subexpressions,
        }
    }

    /// Produces a deterministic fingerprint used by optimizers/CSE.
    pub(crate) fn fingerprint(&self) -> String {
        match self {
            Expr::One => "1".to_string(),
            Expr::Var(index) => format!("v{index}"),
            Expr::Eml(lhs, rhs) => format!("e({},{})", lhs.fingerprint(), rhs.fingerprint()),
        }
    }

    fn to_rpn(&self, out: &mut Vec<Token>) {
        match self {
            Expr::One => out.push(Token::One),
            Expr::Var(index) => out.push(Token::Var(*index)),
            Expr::Eml(lhs, rhs) => {
                lhs.to_rpn(out);
                rhs.to_rpn(out);
                out.push(Token::Eml);
            }
        }
    }
}

/// Evaluates RPN tokens over complex variables with explicit policy.
pub fn eval_rpn_complex_with_policy(
    tokens: &[Token],
    vars: &[Complex64],
    policy: &EvalPolicy,
) -> Result<Complex64, EmlError> {
    let mut stack = Vec::<Complex64>::with_capacity(tokens.len());
    for token in tokens {
        match token {
            Token::One => stack.push(Complex64::new(1.0, 0.0)),
            Token::Var(index) => {
                stack.push(vars.get(*index).copied().ok_or(EmlError::MissingVariable {
                    index: *index,
                    arity: vars.len(),
                })?)
            }
            Token::Eml => {
                let rhs = stack.pop().ok_or(EmlError::StackUnderflow)?;
                let lhs = stack.pop().ok_or(EmlError::StackUnderflow)?;
                stack.push(eml_complex_with_policy(lhs, rhs, policy)?);
            }
        }
    }

    if stack.len() != 1 {
        return Err(EmlError::StackNotSingleton { len: stack.len() });
    }
    Ok(stack[0])
}

/// Evaluates RPN tokens over complex variables with default policy.
pub fn eval_rpn_complex(tokens: &[Token], vars: &[Complex64]) -> Result<Complex64, EmlError> {
    eval_rpn_complex_with_policy(tokens, vars, &EvalPolicy::default())
}

/// Evaluates RPN tokens over real inputs with explicit policy.
pub fn eval_rpn_real_with_policy(
    tokens: &[Token],
    vars: &[f64],
    imag_tolerance: f64,
    policy: &EvalPolicy,
) -> Result<f64, EmlError> {
    let vars_complex: Vec<Complex64> = vars.iter().map(|v| Complex64::new(*v, 0.0)).collect();
    let out = eval_rpn_complex_with_policy(tokens, &vars_complex, policy)?;
    if out.im.abs() > imag_tolerance {
        return Err(EmlError::NonRealOutput {
            imag: out.im,
            tolerance: imag_tolerance,
        });
    }
    Ok(out.re)
}

/// Evaluates RPN tokens over real inputs with default policy.
pub fn eval_rpn_real(tokens: &[Token], vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
    eval_rpn_real_with_policy(tokens, vars, imag_tolerance, &EvalPolicy::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpn_matches_tree_evaluation() {
        let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::var(1));
        let tokens = expr.to_rpn_vec();
        let vars = vec![Complex64::new(0.2, 0.1), Complex64::new(1.4, -0.2)];

        let tree = expr.eval_complex(&vars).unwrap();
        let rpn = eval_rpn_complex(&tokens, &vars).unwrap();
        assert!((tree - rpn).norm() <= 1e-12);
    }

    #[test]
    fn stats_report_shared_subtrees() {
        let repeated = Expr::exp(Expr::var(0));
        let expr = Expr::eml(repeated.clone(), repeated);
        let stats = expr.stats();
        assert!(stats.shared_subexpressions > 0);
        assert!(stats.unique_subexpressions < stats.nodes);
    }
}
