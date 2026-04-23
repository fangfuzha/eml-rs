//! Compatibility wrapper around the standalone `eml-lowering` crate.
//!
//! Parser/lowering implementation lives in `crates/eml-lowering` so it can be
//! reused independently (including `no_std + alloc` scenarios). This module
//! keeps the original `eml-rs` API by converting lowered trees into [`crate::ir::Expr`].

use num_complex::Complex64;

use crate::ir::Expr;
use crate::EmlError;

pub use eml_lowering::{
    batch_cross_entropy_mean_template, batch_cross_entropy_template,
    batch_focal_loss_mean_template, batch_focal_loss_mean_template_with_alpha,
    batch_focal_loss_template, batch_focal_loss_template_with_alpha,
    batch_label_smoothing_cross_entropy_mean_template,
    batch_label_smoothing_cross_entropy_template, batch_softmax_template,
};
pub use eml_lowering::{
    cross_entropy_template, focal_loss_template, focal_loss_template_with_alpha,
    label_smoothing_cross_entropy_template, logsumexp_template, softmax_template,
};
pub use eml_lowering::{delower_to_source, eval_lowered_expr_complex};
pub use eml_lowering::{
    eval_source_expr_complex, parse_source_expr, simplify_source_expr, source_expr_node_count,
    symbolic_derivative,
};
pub use eml_lowering::{LoweredExpr, LoweringError, SourceExpr};

/// Lowers a source expression into standalone EML-only tree (`LoweredExpr`).
pub fn lower_to_lowered_eml(expr: &SourceExpr) -> Result<LoweredExpr, EmlError> {
    Ok(eml_lowering::lower_to_eml(expr)?)
}

/// Converts a source expression into `eml-rs` IR (`Expr`).
pub fn lower_to_eml(expr: &SourceExpr) -> Result<Expr, EmlError> {
    let lowered = lower_to_lowered_eml(expr)?;
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

/// Converts runtime IR (`Expr`) into standalone lowering tree (`LoweredExpr`).
pub fn convert_expr_to_lowered(expr: &Expr) -> LoweredExpr {
    match expr {
        Expr::One => LoweredExpr::one(),
        Expr::Var(index) => LoweredExpr::var(*index),
        Expr::Eml(lhs, rhs) => {
            LoweredExpr::eml(convert_expr_to_lowered(lhs), convert_expr_to_lowered(rhs))
        }
    }
}

/// Raises runtime IR (`Expr`) back to source-level expression via de-lowering.
pub fn raise_expr_to_source(expr: &Expr) -> SourceExpr {
    let lowered = convert_expr_to_lowered(expr);
    delower_to_source(&lowered)
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

    #[test]
    fn raise_expr_to_source_preserves_eval() {
        let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::ln(Expr::var(1)));
        let raised = raise_expr_to_source(&expr);
        let vars = [Complex64::new(0.3, 0.1), Complex64::new(1.7, -0.2)];
        let eml_v = expr.eval_complex(&vars).unwrap();
        let src_v = eval_source_expr_complex(&raised, &vars).unwrap();
        assert!((eml_v - src_v).norm() <= 1e-12);
    }
}
