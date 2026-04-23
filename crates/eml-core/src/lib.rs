#![no_std]
//! no_std core EML primitives and numeric policies.
//!
//! This crate provides the runtime kernel:
//! `eml(x, y) = exp(x) - ln(y)`.

use core::f64::consts::PI;
use core::fmt::{Display, Formatter};

use num_complex::Complex64;

/// Core error type emitted by numeric kernels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmlCoreError {
    /// Mathematical domain error.
    Domain(&'static str),
    /// Non-finite input rejected under strict mode.
    NonFiniteInput(&'static str),
    /// Non-finite output rejected under strict mode.
    NonFiniteOutput(&'static str),
}

impl Display for EmlCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EmlCoreError::Domain(msg) => write!(f, "domain error: {msg}"),
            EmlCoreError::NonFiniteInput(msg) => write!(f, "non-finite input: {msg}"),
            EmlCoreError::NonFiniteOutput(msg) => write!(f, "non-finite output: {msg}"),
        }
    }
}

/// Logarithm branch selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogBranchPolicy {
    /// Native principal branch from `Complex64::ln()`.
    Principal,
    /// Real-axis corrected branch:
    /// - near-positive real => zero imaginary part
    /// - near-negative real => `+pi` imaginary part
    CorrectedReal,
}

/// Non-finite input/output handling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialValuePolicy {
    /// Reject NaN/Inf inputs and outputs with [`EmlCoreError`].
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
/// `log(0)` is rejected under strict mode, and mapped to `-inf+0i` under
/// propagate mode to keep symbolic expansions evaluable.
pub fn log_complex_with_policy(
    y: Complex64,
    policy: &EvalPolicy,
) -> Result<Complex64, EmlCoreError> {
    if y == Complex64::new(0.0, 0.0) {
        if policy.special_values == SpecialValuePolicy::Propagate {
            return Ok(Complex64::new(f64::NEG_INFINITY, 0.0));
        }
        return Err(EmlCoreError::Domain("log(0) is undefined"));
    }

    let mut out = y.ln();
    if policy.log_branch == LogBranchPolicy::CorrectedReal && y.im.abs() <= policy.near_real_epsilon
    {
        if y.re > 0.0 {
            out = Complex64::new(libm::log(y.re), 0.0);
        } else if y.re < 0.0 {
            out = Complex64::new(libm::log(-y.re), PI);
        }
    }
    Ok(out)
}

/// Evaluates `eml(x, y) = exp(x) - ln(y)` for complex inputs with policy.
pub fn eml_complex_with_policy(
    x: Complex64,
    y: Complex64,
    policy: &EvalPolicy,
) -> Result<Complex64, EmlCoreError> {
    if policy.special_values == SpecialValuePolicy::Strict {
        if !is_finite_complex(x) {
            return Err(EmlCoreError::NonFiniteInput("x is not finite"));
        }
        if !is_finite_complex(y) {
            return Err(EmlCoreError::NonFiniteInput("y is not finite"));
        }
    }

    let out = x.exp() - log_complex_with_policy(y, policy)?;
    if policy.special_values == SpecialValuePolicy::Strict && !is_finite_complex(out) {
        return Err(EmlCoreError::NonFiniteOutput(
            "eml_complex produced non-finite value",
        ));
    }
    Ok(out)
}

/// Default complex EML with strict finite checks and principal branch.
pub fn eml_complex(x: Complex64, y: Complex64) -> Result<Complex64, EmlCoreError> {
    eml_complex_with_policy(x, y, &EvalPolicy::default())
}

/// Evaluates real `eml(x, y) = exp(x) - ln(y)` with policy.
pub fn eml_real_with_policy(x: f64, y: f64, policy: &EvalPolicy) -> Result<f64, EmlCoreError> {
    if y <= 0.0 {
        return Err(EmlCoreError::Domain("real log(y) requires y > 0"));
    }

    if policy.special_values == SpecialValuePolicy::Strict {
        if !x.is_finite() {
            return Err(EmlCoreError::NonFiniteInput("x is not finite"));
        }
        if !y.is_finite() {
            return Err(EmlCoreError::NonFiniteInput("y is not finite"));
        }
    }

    let out = libm::exp(x) - libm::log(y);
    if policy.special_values == SpecialValuePolicy::Strict && !out.is_finite() {
        return Err(EmlCoreError::NonFiniteOutput(
            "eml_real produced non-finite value",
        ));
    }
    Ok(out)
}

/// Default real EML with strict finite checks.
pub fn eml_real(x: f64, y: f64) -> Result<f64, EmlCoreError> {
    eml_real_with_policy(x, y, &EvalPolicy::default())
}

#[cfg(test)]
extern crate std;

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
        assert!(matches!(err, EmlCoreError::NonFiniteInput(_)));
    }
}
