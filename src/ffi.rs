//! C ABI exports for embedding `eml-rs` into non-Rust runtimes.
//!
//! The ABI is intentionally small:
//! - evaluate real EML;
//! - evaluate complex EML;
//! - optional policy-controlled complex evaluation.

use num_complex::Complex64;

use crate::core::{
    eml_complex, eml_complex_with_policy, eml_real, EvalPolicy, LogBranchPolicy, SpecialValuePolicy,
};

/// C-compatible complex number representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmlComplexC {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

/// C-compatible policy structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmlEvalPolicyC {
    /// 0 => principal, 1 => corrected-real.
    pub log_branch: u8,
    /// 0 => strict, 1 => propagate.
    pub special_values: u8,
    /// Near-real epsilon used by corrected-real branch mode.
    pub near_real_epsilon: f64,
}

impl Default for EmlEvalPolicyC {
    fn default() -> Self {
        Self {
            log_branch: 0,
            special_values: 0,
            near_real_epsilon: 1e-12,
        }
    }
}

impl From<EmlEvalPolicyC> for EvalPolicy {
    fn from(value: EmlEvalPolicyC) -> Self {
        let log_branch = if value.log_branch == 1 {
            LogBranchPolicy::CorrectedReal
        } else {
            LogBranchPolicy::Principal
        };
        let special_values = if value.special_values == 1 {
            SpecialValuePolicy::Propagate
        } else {
            SpecialValuePolicy::Strict
        };
        EvalPolicy {
            log_branch,
            special_values,
            near_real_epsilon: value.near_real_epsilon,
        }
    }
}

/// Success status code.
pub const EML_FFI_OK: i32 = 0;
/// Generic evaluation failure.
pub const EML_FFI_EVAL_ERROR: i32 = 1;
/// Null output pointer passed to API.
pub const EML_FFI_NULL_OUT: i32 = 2;

/// Evaluates real `eml(x, y)` with default policy.
///
/// # Safety
/// - `out` must be valid for writes of one `f64`.
#[no_mangle]
pub unsafe extern "C" fn eml_rs_eval_real(x: f64, y: f64, out: *mut f64) -> i32 {
    if out.is_null() {
        return EML_FFI_NULL_OUT;
    }

    match eml_real(x, y) {
        Ok(v) => {
            // SAFETY: caller guarantees `out` points to valid writable memory.
            unsafe { *out = v };
            EML_FFI_OK
        }
        Err(_) => EML_FFI_EVAL_ERROR,
    }
}

/// Evaluates complex `eml(x, y)` with default policy.
///
/// # Safety
/// - `out` must be valid for writes of one [`EmlComplexC`].
#[no_mangle]
pub unsafe extern "C" fn eml_rs_eval_complex(
    x_re: f64,
    x_im: f64,
    y_re: f64,
    y_im: f64,
    out: *mut EmlComplexC,
) -> i32 {
    if out.is_null() {
        return EML_FFI_NULL_OUT;
    }

    let x = Complex64::new(x_re, x_im);
    let y = Complex64::new(y_re, y_im);
    match eml_complex(x, y) {
        Ok(v) => {
            // SAFETY: caller guarantees `out` points to valid writable memory.
            unsafe {
                *out = EmlComplexC { re: v.re, im: v.im };
            }
            EML_FFI_OK
        }
        Err(_) => EML_FFI_EVAL_ERROR,
    }
}

/// Evaluates complex `eml(x, y)` with explicit policy.
///
/// # Safety
/// - `out` must be valid for writes of one [`EmlComplexC`].
#[no_mangle]
pub unsafe extern "C" fn eml_rs_eval_complex_with_policy(
    x_re: f64,
    x_im: f64,
    y_re: f64,
    y_im: f64,
    policy: EmlEvalPolicyC,
    out: *mut EmlComplexC,
) -> i32 {
    if out.is_null() {
        return EML_FFI_NULL_OUT;
    }

    let x = Complex64::new(x_re, x_im);
    let y = Complex64::new(y_re, y_im);
    match eml_complex_with_policy(x, y, &EvalPolicy::from(policy)) {
        Ok(v) => {
            // SAFETY: caller guarantees `out` points to valid writable memory.
            unsafe {
                *out = EmlComplexC { re: v.re, im: v.im };
            }
            EML_FFI_OK
        }
        Err(_) => EML_FFI_EVAL_ERROR,
    }
}
