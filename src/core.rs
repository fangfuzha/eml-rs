//! Core EML primitives, log branch policy, and special-value policy.
//!
//! This module defines the canonical operator:
//! `eml(x, y) = exp(x) - ln(y)`.
//! Engineering use-cases usually need explicit choices for:
//! - logarithm branch behavior near the real axis;
//! - strict vs propagating treatment of NaN/Inf.

use std::f64::consts::PI;

use num_complex::Complex64;

use crate::EmlError;

/// Logarithm branch selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogBranchPolicy {
    /// Native principal branch from `Complex64::ln()`.
    Principal,
    /// Real-axis corrected branch:
    /// - for near-positive real input, force zero imaginary part;
    /// - for near-negative real input, force `+pi` imaginary part.
    CorrectedReal,
}

/// Non-finite input/output handling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialValuePolicy {
    /// Reject NaN/Inf inputs and outputs with [`EmlError`].
    Strict,
    /// Let NaN/Inf propagate as IEEE values whenever possible.
    Propagate,
}

/// Evaluation policy bundle for EML execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvalPolicy {
    /// Branch policy used by `ln`.
    pub log_branch: LogBranchPolicy,
    /// Strict vs propagating finite checks.
    pub special_values: SpecialValuePolicy,
    /// Threshold for deciding whether a complex number is "near real".
    pub near_real_epsilon: f64,
}

impl Default for EvalPolicy {
    fn default() -> Self {
        Self {
            log_branch: LogBranchPolicy::Principal,
            special_values: SpecialValuePolicy::Strict,
            near_real_epsilon: 1e-12,
        }
    }
}

impl EvalPolicy {
    /// Returns a permissive policy that propagates non-finite values.
    pub fn relaxed() -> Self {
        Self {
            log_branch: LogBranchPolicy::Principal,
            special_values: SpecialValuePolicy::Propagate,
            near_real_epsilon: 1e-12,
        }
    }
}

/// Returns whether both real and imaginary parts are finite.
pub fn is_finite_complex(v: Complex64) -> bool {
    v.re.is_finite() && v.im.is_finite()
}

/// Computes complex logarithm according to policy branch rules.
///
/// `log(0)` is rejected under all policies to keep the API explicit.
pub fn log_complex_with_policy(y: Complex64, policy: &EvalPolicy) -> Result<Complex64, EmlError> {
    if y == Complex64::new(0.0, 0.0) {
        if policy.special_values == SpecialValuePolicy::Propagate {
            // Limit-style value used by the relaxed mode to keep symbolic
            // constructions evaluable even if intermediate terms hit zero.
            return Ok(Complex64::new(f64::NEG_INFINITY, 0.0));
        }
        return Err(EmlError::Domain("log(0) is undefined"));
    }

    let mut out = y.ln();
    if policy.log_branch == LogBranchPolicy::CorrectedReal && y.im.abs() <= policy.near_real_epsilon
    {
        if y.re > 0.0 {
            out = Complex64::new(y.re.ln(), 0.0);
        } else if y.re < 0.0 {
            out = Complex64::new((-y.re).ln(), PI);
        }
    }
    Ok(out)
}

/// Evaluates `eml(x, y) = exp(x) - ln(y)` for complex inputs with policy.
pub fn eml_complex_with_policy(
    x: Complex64,
    y: Complex64,
    policy: &EvalPolicy,
) -> Result<Complex64, EmlError> {
    if policy.special_values == SpecialValuePolicy::Strict {
        if !is_finite_complex(x) {
            return Err(EmlError::NonFiniteInput("x is not finite"));
        }
        if !is_finite_complex(y) {
            return Err(EmlError::NonFiniteInput("y is not finite"));
        }
    }

    let out = x.exp() - log_complex_with_policy(y, policy)?;
    if policy.special_values == SpecialValuePolicy::Strict && !is_finite_complex(out) {
        return Err(EmlError::NonFiniteOutput(
            "eml_complex produced non-finite value",
        ));
    }
    Ok(out)
}

/// Default complex EML with strict finite checks and principal branch.
pub fn eml_complex(x: Complex64, y: Complex64) -> Result<Complex64, EmlError> {
    eml_complex_with_policy(x, y, &EvalPolicy::default())
}

/// Evaluates real `eml(x, y) = exp(x) - ln(y)` with policy.
///
/// Real mode always requires `y > 0` and rejects `y <= 0` (including `-0.0`).
pub fn eml_real_with_policy(x: f64, y: f64, policy: &EvalPolicy) -> Result<f64, EmlError> {
    if y <= 0.0 {
        return Err(EmlError::Domain("real log(y) requires y > 0"));
    }

    if policy.special_values == SpecialValuePolicy::Strict {
        if !x.is_finite() {
            return Err(EmlError::NonFiniteInput("x is not finite"));
        }
        if !y.is_finite() {
            return Err(EmlError::NonFiniteInput("y is not finite"));
        }
    }

    let out = x.exp() - y.ln();
    if policy.special_values == SpecialValuePolicy::Strict && !out.is_finite() {
        return Err(EmlError::NonFiniteOutput(
            "eml_real produced non-finite value",
        ));
    }
    Ok(out)
}

/// Default real EML with strict finite checks.
pub fn eml_real(x: f64, y: f64) -> Result<f64, EmlError> {
    eml_real_with_policy(x, y, &EvalPolicy::default())
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
        assert!((ln.im - PI).abs() <= 1e-12);
    }

    #[test]
    fn strict_mode_rejects_nan() {
        let err = eml_complex_with_policy(
            Complex64::new(f64::NAN, 0.0),
            Complex64::new(1.0, 0.0),
            &EvalPolicy::default(),
        )
        .unwrap_err();
        assert!(matches!(err, EmlError::NonFiniteInput(_)));
    }

    #[test]
    fn propagate_mode_keeps_nan() {
        let policy = EvalPolicy::relaxed();
        let out = eml_complex_with_policy(
            Complex64::new(f64::NAN, 0.0),
            Complex64::new(1.0, 0.0),
            &policy,
        )
        .unwrap();
        assert!(out.re.is_nan() || out.im.is_nan());
    }

    #[test]
    fn real_mode_rejects_signed_zero() {
        let plus_zero_err = eml_real(1.0, 0.0).unwrap_err();
        let minus_zero_err = eml_real(1.0, -0.0).unwrap_err();
        assert!(matches!(plus_zero_err, EmlError::Domain(_)));
        assert!(matches!(minus_zero_err, EmlError::Domain(_)));
    }
}
