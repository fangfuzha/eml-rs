//! Verification utilities for expression-vs-reference comparisons.

use num_complex::Complex64;

use crate::core::{is_finite_complex, EvalPolicy};
use crate::ir::Expr;

/// Aggregate verification result.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Number of attempted samples.
    pub total: usize,
    /// Samples with error `<= tolerance`.
    pub passed: usize,
    /// Samples that failed or errored.
    pub failed: usize,
    /// Maximum absolute error observed across all successful evaluations.
    pub max_abs_error: f64,
}

impl VerificationReport {
    /// Returns true when every sample passed.
    pub fn all_passed(&self) -> bool {
        self.total > 0 && self.failed == 0
    }
}

/// Per-backend summary for cross-backend reporting.
#[derive(Debug, Clone)]
pub struct BackendComparison {
    /// Backend label.
    pub backend: String,
    /// Pairwise report against the same target expression.
    pub report: VerificationReport,
}

/// Verifies complex outputs against a user-provided complex reference function.
pub fn verify_against_complex_ref_with_policy(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    policy: &EvalPolicy,
    reference: impl Fn(&[Complex64]) -> Complex64,
) -> VerificationReport {
    let mut report = VerificationReport {
        total: 0,
        passed: 0,
        failed: 0,
        max_abs_error: 0.0,
    };

    for vars in samples {
        report.total += 1;
        let expected = reference(vars);
        if !is_finite_complex(expected) {
            report.failed += 1;
            continue;
        }

        let actual = match expr.eval_complex_with_policy(vars, policy) {
            Ok(v) => v,
            Err(_) => {
                report.failed += 1;
                continue;
            }
        };

        let err = (actual - expected).norm();
        report.max_abs_error = report.max_abs_error.max(err);
        if err <= tolerance {
            report.passed += 1;
        } else {
            report.failed += 1;
        }
    }

    report
}

/// Verifies complex outputs against a reference function under default policy.
pub fn verify_against_complex_ref(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    reference: impl Fn(&[Complex64]) -> Complex64,
) -> VerificationReport {
    verify_against_complex_ref_with_policy(
        expr,
        samples,
        tolerance,
        &EvalPolicy::default(),
        reference,
    )
}

/// Verifies real outputs against a user-provided real reference function.
pub fn verify_against_real_ref_with_policy(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
    policy: &EvalPolicy,
    reference: impl Fn(&[f64]) -> f64,
) -> VerificationReport {
    let mut report = VerificationReport {
        total: 0,
        passed: 0,
        failed: 0,
        max_abs_error: 0.0,
    };

    for vars in samples {
        report.total += 1;
        let expected = reference(vars);
        if !expected.is_finite() {
            report.failed += 1;
            continue;
        }

        let actual = match expr.eval_real_with_policy(vars, imag_tolerance, policy) {
            Ok(v) => v,
            Err(_) => {
                report.failed += 1;
                continue;
            }
        };

        let err = (actual - expected).abs();
        report.max_abs_error = report.max_abs_error.max(err);
        if err <= tolerance {
            report.passed += 1;
        } else {
            report.failed += 1;
        }
    }

    report
}

/// Verifies real outputs against a reference function under default policy.
pub fn verify_against_real_ref(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
    reference: impl Fn(&[f64]) -> f64,
) -> VerificationReport {
    verify_against_real_ref_with_policy(
        expr,
        samples,
        imag_tolerance,
        tolerance,
        &EvalPolicy::default(),
        reference,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ir::Expr;

    #[test]
    fn exp_identity_passes_verification() {
        let expr = Expr::exp(Expr::var(0));
        let samples = vec![
            vec![Complex64::new(-2.0, 0.0)],
            vec![Complex64::new(-0.5, 0.2)],
            vec![Complex64::new(0.0, 0.0)],
            vec![Complex64::new(0.8, -0.3)],
        ];

        let report = verify_against_complex_ref(&expr, &samples, 1e-12, |vars| vars[0].exp());
        assert!(report.all_passed(), "{report:?}");
    }
}
