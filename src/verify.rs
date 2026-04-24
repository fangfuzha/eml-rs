//! Verification utilities for expression-vs-reference comparisons.

use std::thread;

use num_complex::Complex64;

use crate::core::{is_finite_complex, EvalPolicy};
use crate::ir::Expr;

/// Aggregate verification result.
#[derive(Debug, Clone, PartialEq)]
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
    /// Creates an empty report ready for accumulation.
    pub fn empty() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            max_abs_error: 0.0,
        }
    }

    /// Returns true when every sample passed.
    pub fn all_passed(&self) -> bool {
        self.total > 0 && self.failed == 0
    }

    /// Merges a per-chunk report into the current aggregate.
    pub fn merge(&mut self, other: &Self) {
        self.total += other.total;
        self.passed += other.passed;
        self.failed += other.failed;
        self.max_abs_error = self.max_abs_error.max(other.max_abs_error);
    }
}

impl Default for VerificationReport {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sample-level parallelism settings for verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyParallelism {
    /// Maximum worker count to use for independent sample chunks.
    pub workers: usize,
    /// Minimum samples each worker should receive before parallelizing.
    pub min_samples_per_worker: usize,
}

impl VerifyParallelism {
    /// Chooses the process-wide default based on available CPU parallelism.
    pub fn auto() -> Self {
        Self::default()
    }

    /// Returns the effective worker count for a concrete batch.
    pub fn effective_workers(self, sample_count: usize) -> usize {
        if sample_count == 0 {
            return 1;
        }

        let workers = self.workers.max(1).min(sample_count);
        let min_samples = self.min_samples_per_worker.max(1);
        if workers <= 1 || sample_count < workers * min_samples {
            1
        } else {
            workers
        }
    }

    fn chunk_size(self, sample_count: usize) -> usize {
        sample_count.div_ceil(self.effective_workers(sample_count))
    }
}

impl Default for VerifyParallelism {
    fn default() -> Self {
        Self {
            workers: thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
            min_samples_per_worker: 64,
        }
    }
}

/// Per-backend summary for cross-backend reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendComparison {
    /// Backend label.
    pub backend: String,
    /// Pairwise report against the same target expression.
    pub report: VerificationReport,
}

fn verify_complex_slice_with_policy<F>(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    policy: &EvalPolicy,
    reference: &F,
) -> VerificationReport
where
    F: Fn(&[Complex64]) -> Complex64 + ?Sized,
{
    let mut report = VerificationReport::empty();

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

fn verify_real_slice_with_policy<F>(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
    policy: &EvalPolicy,
    reference: &F,
) -> VerificationReport
where
    F: Fn(&[f64]) -> f64 + ?Sized,
{
    let mut report = VerificationReport::empty();

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

/// Verifies complex outputs against a user-provided complex reference function.
pub fn verify_against_complex_ref_with_policy(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    policy: &EvalPolicy,
    reference: impl Fn(&[Complex64]) -> Complex64,
) -> VerificationReport {
    verify_complex_slice_with_policy(expr, samples, tolerance, policy, &reference)
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

/// Verifies complex outputs in parallel across independent sample chunks.
pub fn verify_against_complex_ref_parallel_with_policy(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    policy: &EvalPolicy,
    parallelism: VerifyParallelism,
    reference: impl Fn(&[Complex64]) -> Complex64 + Sync,
) -> VerificationReport {
    let workers = parallelism.effective_workers(samples.len());
    if workers <= 1 {
        return verify_complex_slice_with_policy(expr, samples, tolerance, policy, &reference);
    }

    let chunk_size = parallelism.chunk_size(samples.len());
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        let reference = &reference;
        for chunk in samples.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                verify_complex_slice_with_policy(expr, chunk, tolerance, policy, reference)
            }));
        }

        let mut report = VerificationReport::empty();
        for handle in handles {
            report.merge(
                &handle
                    .join()
                    .expect("verify complex worker unexpectedly panicked"),
            );
        }
        report
    })
}

/// Verifies complex outputs in parallel under default policy.
pub fn verify_against_complex_ref_parallel(
    expr: &Expr,
    samples: &[Vec<Complex64>],
    tolerance: f64,
    parallelism: VerifyParallelism,
    reference: impl Fn(&[Complex64]) -> Complex64 + Sync,
) -> VerificationReport {
    verify_against_complex_ref_parallel_with_policy(
        expr,
        samples,
        tolerance,
        &EvalPolicy::default(),
        parallelism,
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
    verify_real_slice_with_policy(expr, samples, imag_tolerance, tolerance, policy, &reference)
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

/// Verifies real outputs in parallel across independent sample chunks.
pub fn verify_against_real_ref_parallel_with_policy(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
    policy: &EvalPolicy,
    parallelism: VerifyParallelism,
    reference: impl Fn(&[f64]) -> f64 + Sync,
) -> VerificationReport {
    let workers = parallelism.effective_workers(samples.len());
    if workers <= 1 {
        return verify_real_slice_with_policy(
            expr,
            samples,
            imag_tolerance,
            tolerance,
            policy,
            &reference,
        );
    }

    let chunk_size = parallelism.chunk_size(samples.len());
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        let reference = &reference;
        for chunk in samples.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                verify_real_slice_with_policy(
                    expr,
                    chunk,
                    imag_tolerance,
                    tolerance,
                    policy,
                    reference,
                )
            }));
        }

        let mut report = VerificationReport::empty();
        for handle in handles {
            report.merge(
                &handle
                    .join()
                    .expect("verify real worker unexpectedly panicked"),
            );
        }
        report
    })
}

/// Verifies real outputs in parallel under default policy.
pub fn verify_against_real_ref_parallel(
    expr: &Expr,
    samples: &[Vec<f64>],
    imag_tolerance: f64,
    tolerance: f64,
    parallelism: VerifyParallelism,
    reference: impl Fn(&[f64]) -> f64 + Sync,
) -> VerificationReport {
    verify_against_real_ref_parallel_with_policy(
        expr,
        samples,
        imag_tolerance,
        tolerance,
        &EvalPolicy::default(),
        parallelism,
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

    #[test]
    fn auto_parallelism_never_returns_zero_workers() {
        let workers = VerifyParallelism::auto().effective_workers(8);
        assert!(workers >= 1);
    }
}
