//! Core compatibility layer over standalone `eml-core`.
//!
//! The actual numeric kernel lives in `crates/eml-core` (`no_std`).
//! This module preserves `eml-rs` error typing and API ergonomics.

use num_complex::Complex64;

use crate::EmlError;

pub use eml_core::is_finite_complex;
pub use eml_core::{EvalPolicy, LogBranchPolicy, SpecialValuePolicy};

/// Computes complex logarithm according to policy branch rules.
pub fn log_complex_with_policy(y: Complex64, policy: &EvalPolicy) -> Result<Complex64, EmlError> {
    eml_core::log_complex_with_policy(y, policy).map_err(EmlError::from)
}

/// Evaluates `eml(x, y) = exp(x) - ln(y)` for complex inputs with policy.
pub fn eml_complex_with_policy(
    x: Complex64,
    y: Complex64,
    policy: &EvalPolicy,
) -> Result<Complex64, EmlError> {
    eml_core::eml_complex_with_policy(x, y, policy).map_err(EmlError::from)
}

/// Default complex EML with strict finite checks and principal branch.
pub fn eml_complex(x: Complex64, y: Complex64) -> Result<Complex64, EmlError> {
    eml_core::eml_complex(x, y).map_err(EmlError::from)
}

/// Evaluates real `eml(x, y) = exp(x) - ln(y)` with policy.
pub fn eml_real_with_policy(x: f64, y: f64, policy: &EvalPolicy) -> Result<f64, EmlError> {
    eml_core::eml_real_with_policy(x, y, policy).map_err(EmlError::from)
}

/// Default real EML with strict finite checks.
pub fn eml_real(x: f64, y: f64) -> Result<f64, EmlError> {
    eml_core::eml_real(x, y).map_err(EmlError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eml_real_matches_definition() {
        let x = 0.7;
        let y = 2.5;
        let got = eml_real(x, y).unwrap();
        let expected = x.exp() - y.ln();
        assert!((got - expected).abs() <= 1e-14);
    }

    #[test]
    fn corrected_real_branch_returns_pi_on_negative_real_axis() {
        let policy = EvalPolicy {
            log_branch: LogBranchPolicy::CorrectedReal,
            ..EvalPolicy::default()
        };
        let ln = log_complex_with_policy(Complex64::new(-2.0, 0.0), &policy).unwrap();
        assert!((ln.re - (2.0f64).ln()).abs() <= 1e-12);
        assert!((ln.im - core::f64::consts::PI).abs() <= 1e-12);
    }
}
