use num_complex::Complex64;

use crate::core::eml_complex;
use crate::EmlError;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    One,
    Var(usize),
    Eml(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    One,
    Var(usize),
    Eml,
}

impl Expr {
    pub fn one() -> Self {
        Self::One
    }

    pub fn var(index: usize) -> Self {
        Self::Var(index)
    }

    pub fn eml(lhs: Expr, rhs: Expr) -> Self {
        Self::Eml(Box::new(lhs), Box::new(rhs))
    }

    pub fn exp(arg: Expr) -> Self {
        Self::eml(arg, Self::one())
    }

    pub fn ln(arg: Expr) -> Self {
        // ln(x) = eml(1, eml(eml(1, x), 1))
        Self::eml(
            Self::one(),
            Self::eml(Self::eml(Self::one(), arg), Self::one()),
        )
    }

    pub fn eval_complex(&self, vars: &[Complex64]) -> Result<Complex64, EmlError> {
        match self {
            Expr::One => Ok(Complex64::new(1.0, 0.0)),
            Expr::Var(index) => vars.get(*index).copied().ok_or(EmlError::MissingVariable {
                index: *index,
                arity: vars.len(),
            }),
            Expr::Eml(lhs, rhs) => {
                let l = lhs.eval_complex(vars)?;
                let r = rhs.eval_complex(vars)?;
                eml_complex(l, r)
            }
        }
    }

    pub fn eval_real(&self, vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
        let vars_complex: Vec<Complex64> = vars.iter().map(|v| Complex64::new(*v, 0.0)).collect();
        let out = self.eval_complex(&vars_complex)?;
        if out.im.abs() > imag_tolerance {
            return Err(EmlError::NonRealOutput {
                imag: out.im,
                tolerance: imag_tolerance,
            });
        }
        Ok(out.re)
    }

    pub fn to_rpn_vec(&self) -> Vec<Token> {
        let mut out = Vec::new();
        self.to_rpn(&mut out);
        out
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

pub fn eval_rpn_complex(tokens: &[Token], vars: &[Complex64]) -> Result<Complex64, EmlError> {
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
                stack.push(eml_complex(lhs, rhs)?);
            }
        }
    }

    if stack.len() != 1 {
        return Err(EmlError::StackNotSingleton { len: stack.len() });
    }
    Ok(stack[0])
}

pub fn eval_rpn_real(tokens: &[Token], vars: &[f64], imag_tolerance: f64) -> Result<f64, EmlError> {
    let vars_complex: Vec<Complex64> = vars.iter().map(|v| Complex64::new(*v, 0.0)).collect();
    let out = eval_rpn_complex(tokens, &vars_complex)?;
    if out.im.abs() > imag_tolerance {
        return Err(EmlError::NonRealOutput {
            imag: out.im,
            tolerance: imag_tolerance,
        });
    }
    Ok(out.re)
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
}
