use eml_rs::core::{
    eml_complex_with_policy, eml_real_with_policy, log_complex_with_policy, EvalPolicy,
    LogBranchPolicy, SpecialValuePolicy,
};
use eml_rs::EmlError;
use num_complex::Complex64;

#[test]
fn strict_rejects_infinite_inputs() {
    let policy = EvalPolicy::default();
    let err = eml_complex_with_policy(
        Complex64::new(f64::INFINITY, 0.0),
        Complex64::new(1.0, 0.0),
        &policy,
    )
    .unwrap_err();
    assert!(matches!(err, EmlError::NonFiniteInput(_)));
}

#[test]
fn propagate_allows_nan_output() {
    let policy = EvalPolicy {
        special_values: SpecialValuePolicy::Propagate,
        ..EvalPolicy::default()
    };
    let out = eml_complex_with_policy(
        Complex64::new(f64::NAN, 0.0),
        Complex64::new(1.0, 0.0),
        &policy,
    )
    .unwrap();
    assert!(out.re.is_nan() || out.im.is_nan());
}

#[test]
fn corrected_real_branch_stabilizes_positive_axis() {
    let policy = EvalPolicy {
        log_branch: LogBranchPolicy::CorrectedReal,
        ..EvalPolicy::default()
    };
    let ln = log_complex_with_policy(Complex64::new(3.5, 1e-15), &policy).unwrap();
    assert!(ln.im.abs() <= 1e-12);
}

#[test]
fn real_path_rejects_negative_zero() {
    let err = eml_real_with_policy(0.0, -0.0, &EvalPolicy::default()).unwrap_err();
    assert!(matches!(err, EmlError::Domain(_)));
}
