use num_complex::Complex64;

use crate::core::is_finite_complex;
use crate::ir::Expr;

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub max_abs_error: f64,
}

impl VerificationReport {
    pub fn all_passed(&self) -> bool {
        self.total > 0 && self.failed == 0
    }
}

pub fn verify_against_complex_ref(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
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

        let actual = match expr.eval_complex(vars) {
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

pub fn verify_against_real_ref(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
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

        let actual = match expr.eval_real(vars, imag_tolerance) {
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
