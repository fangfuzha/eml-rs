//! Compatibility wrapper around the standalone `eml-lowering` crate.
//!
//! Parser/lowering implementation lives in `crates/eml-lowering` so it can be
//! reused independently (including `no_std + alloc` scenarios). This module
//! keeps the original `eml-rs` API by converting lowered trees into [`crate::ir::Expr`].

use num_complex::Complex64;

use crate::ir::Expr;
use crate::EmlError;

pub use eml_lowering::{eval_source_expr_complex, parse_source_expr};
pub use eml_lowering::{LoweredExpr, LoweringError, SourceExpr};

/// Converts a source expression into `eml-rs` IR (`Expr`).
pub fn lower_to_eml(expr: &SourceExpr) -> Result<Expr, EmlError> {
    let lowered = eml_lowering::lower_to_eml(expr)?;
    Ok(convert_lowered_expr(&lowered))
}

/// Converts standalone lowering tree into the runtime IR.
pub fn convert_lowered_expr(expr: &LoweredExpr) -> Expr {
    match expr {
        LoweredExpr::One => Expr::one(),
        LoweredExpr::Var(index) => Expr::var(*index),
        LoweredExpr::Eml(lhs, rhs) => {
            Expr::eml(convert_lowered_expr(lhs), convert_lowered_expr(rhs))
        }
    }
}

/// Convenience adapter that parses and lowers in one step.
pub fn parse_and_lower(input: &str) -> Result<Expr, EmlError> {
    let source = parse_source_expr(input)?;
    lower_to_eml(&source)
}

/// Evaluates parsed source expression through standalone lowering crate.
///
/// This helper is kept here for ergonomic parity with the previous API.
pub fn eval_source_complex(expr: &SourceExpr, vars: &[Complex64]) -> Result<Complex64, EmlError> {
    Ok(eval_source_expr_complex(expr, vars)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_lower_roundtrip() {
        let expr = parse_and_lower("exp(x0) - log(x1)").unwrap();
        let vars = [Complex64::new(0.4, -0.2), Complex64::new(1.3, 0.1)];
        let val = expr.eval_complex(&vars).unwrap();
        assert!(val.re.is_finite() && val.im.is_finite());
    }
}
