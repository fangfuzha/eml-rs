use eml_core::EmlCoreError;
use eml_rs::error::{EmlError, EmlErrorCode};
use eml_rs::ffi::{
    eml_rs_eval_complex, eml_rs_eval_complex_with_policy, eml_rs_eval_real, EmlComplexC,
    EmlEvalPolicyC, EML_FFI_EVAL_ERROR, EML_FFI_NULL_OUT, EML_FFI_OK,
};
use eml_rs::lowering::LoweringError;
use num_complex::Complex64;

#[test]
fn ffi_real_and_complex_paths_report_status_codes() {
    let mut real_out = 0.0;
    // SAFETY: valid output pointer for one `f64`.
    let real_status = unsafe { eml_rs_eval_real(0.25, 2.0, &mut real_out) };
    assert_eq!(real_status, EML_FFI_OK);
    assert!((real_out - (0.25_f64.exp() - 2.0_f64.ln())).abs() <= 1e-12);

    // SAFETY: the API explicitly accepts null and reports a status code.
    let null_status = unsafe { eml_rs_eval_real(0.25, 2.0, core::ptr::null_mut()) };
    assert_eq!(null_status, EML_FFI_NULL_OUT);

    // SAFETY: valid output pointer for one `f64`.
    let domain_status = unsafe { eml_rs_eval_real(0.25, 0.0, &mut real_out) };
    assert_eq!(domain_status, EML_FFI_EVAL_ERROR);

    let mut complex_out = EmlComplexC { re: 0.0, im: 0.0 };
    // SAFETY: valid output pointer for one `EmlComplexC`.
    let complex_status = unsafe { eml_rs_eval_complex(0.25, 0.0, 2.0, 0.0, &mut complex_out) };
    assert_eq!(complex_status, EML_FFI_OK);
    let expected = Complex64::new(0.25, 0.0).exp() - Complex64::new(2.0, 0.0).ln();
    assert!((complex_out.re - expected.re).abs() <= 1e-12);
    assert!((complex_out.im - expected.im).abs() <= 1e-12);
}

#[test]
fn ffi_policy_mapping_controls_corrected_real_branch() {
    let mut out = EmlComplexC { re: 0.0, im: 0.0 };
    let policy = EmlEvalPolicyC {
        log_branch: 1,
        special_values: 0,
        near_real_epsilon: 1e-12,
    };

    // SAFETY: valid output pointer for one `EmlComplexC`.
    let status = unsafe { eml_rs_eval_complex_with_policy(0.0, 0.0, -2.0, 0.0, policy, &mut out) };
    assert_eq!(status, EML_FFI_OK);
    assert!((out.re - (1.0 - (2.0_f64).ln())).abs() <= 1e-12);
    assert!((out.im + core::f64::consts::PI).abs() <= 1e-12);
}

#[test]
fn ffi_complex_reports_null_and_eval_errors() {
    let mut out = EmlComplexC { re: 0.0, im: 0.0 };
    // SAFETY: the API explicitly accepts null and reports a status code.
    let null_status = unsafe { eml_rs_eval_complex(0.0, 0.0, 1.0, 0.0, core::ptr::null_mut()) };
    assert_eq!(null_status, EML_FFI_NULL_OUT);

    // SAFETY: valid output pointer for one `EmlComplexC`.
    let eval_error = unsafe { eml_rs_eval_complex(0.0, 0.0, 0.0, 0.0, &mut out) };
    assert_eq!(eval_error, EML_FFI_EVAL_ERROR);

    let default_policy = EmlEvalPolicyC::default();
    assert_eq!(default_policy.log_branch, 0);
    assert_eq!(default_policy.special_values, 0);
    assert!((default_policy.near_real_epsilon - 1e-12).abs() <= f64::EPSILON);
}

#[test]
fn error_codes_and_conversions_are_stable() {
    let err = EmlError::StackNotSingleton { len: 3 };
    let diag = err.diagnostic();
    assert_eq!(diag.code, EmlErrorCode::StackNotSingleton);
    assert_eq!(diag.code.name(), "STACK_NOT_SINGLETON");
    assert_eq!(diag.category, "execution");
    assert_eq!(diag.code.to_string(), "STACK_NOT_SINGLETON(2001)");

    let core_domain: EmlError = EmlCoreError::Domain("bad input").into();
    assert_eq!(core_domain.code(), EmlErrorCode::Domain);
    assert!(core_domain.to_string().contains("bad input"));

    let core_non_finite: EmlError = EmlCoreError::NonFiniteOutput("overflowed").into();
    assert_eq!(core_non_finite.code(), EmlErrorCode::NonFiniteOutput);

    let lowering_parse: EmlError = LoweringError::Parse("bad syntax".to_string()).into();
    assert_eq!(lowering_parse.code(), EmlErrorCode::Parse);

    let lowering_missing: EmlError = LoweringError::MissingVariable { index: 4, arity: 2 }.into();
    assert_eq!(lowering_missing.code(), EmlErrorCode::MissingVariable);
    assert!(lowering_missing.to_string().contains("index 4"));
}

#[test]
fn error_display_and_categories_cover_all_variants() {
    let errors = vec![
        (
            EmlError::Domain("bad domain"),
            EmlErrorCode::Domain,
            "semantic",
        ),
        (
            EmlError::MissingVariable { index: 1, arity: 0 },
            EmlErrorCode::MissingVariable,
            "semantic",
        ),
        (
            EmlError::NonFiniteInput("nan"),
            EmlErrorCode::NonFiniteInput,
            "semantic",
        ),
        (
            EmlError::NonFiniteOutput("inf"),
            EmlErrorCode::NonFiniteOutput,
            "semantic",
        ),
        (
            EmlError::StackUnderflow,
            EmlErrorCode::StackUnderflow,
            "execution",
        ),
        (
            EmlError::StackNotSingleton { len: 2 },
            EmlErrorCode::StackNotSingleton,
            "execution",
        ),
        (
            EmlError::NonRealOutput {
                imag: 0.25,
                tolerance: 1e-6,
            },
            EmlErrorCode::NonRealOutput,
            "semantic",
        ),
        (
            EmlError::Parse("bad parse".to_string()),
            EmlErrorCode::Parse,
            "compile",
        ),
        (
            EmlError::Unsupported("todo"),
            EmlErrorCode::Unsupported,
            "compile",
        ),
        (EmlError::Overflow("big"), EmlErrorCode::Overflow, "compile"),
    ];

    for (err, code, category) in errors {
        assert_eq!(err.code(), code);
        assert_eq!(err.category(), category);
        let diag = err.diagnostic();
        assert_eq!(diag.code, code);
        assert_eq!(diag.category, category);
        assert!(!err.to_string().is_empty());
    }
}
