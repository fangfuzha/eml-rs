//! eml-rs: Engineering skeleton for EML-based expression execution.
//!
//! The crate is organized into:
//! - [`core`]: numeric EML primitives and evaluation policy.
//! - [`ir`]: tree IR, RPN conversion/eval, and IR statistics.
//! - [`bytecode`]: register bytecode compiler/executor with CSE+const-fold.
//! - [`lowering`]: compatibility wrapper for the standalone parser/lowering crate.
//! - [`opt`]: rewrite rules and cost model utilities.
//! - [`verify`]: numeric cross-check helpers.
//! - [`ffi`]: C ABI exports for embedding.

use std::fmt::{Display, Formatter};

pub mod bytecode;
pub mod core;
pub mod ffi;
pub mod ir;
pub mod lowering;
pub mod opt;
pub mod verify;

/// Unified error type for parser, compiler, and evaluator paths.
#[derive(Debug, Clone, PartialEq)]
pub enum EmlError {
    /// Domain errors (for example `log(0)`).
    Domain(&'static str),
    /// Variable index is out of bounds for provided inputs.
    MissingVariable { index: usize, arity: usize },
    /// Strict policy rejected non-finite input.
    NonFiniteInput(&'static str),
    /// Strict policy rejected non-finite output.
    NonFiniteOutput(&'static str),
    /// RPN/stack-machine underflow.
    StackUnderflow,
    /// RPN/stack-machine did not end with exactly one output.
    StackNotSingleton { len: usize },
    /// A real-valued path produced a significant imaginary residual.
    NonRealOutput { imag: f64, tolerance: f64 },
    /// Parsing failed for a source expression.
    Parse(String),
    /// Feature is intentionally unsupported in current lowering path.
    Unsupported(&'static str),
}

impl Display for EmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmlError::Domain(msg) => write!(f, "domain error: {msg}"),
            EmlError::MissingVariable { index, arity } => {
                write!(f, "missing variable at index {index}, arity is {arity}")
            }
            EmlError::NonFiniteInput(msg) => write!(f, "non-finite input: {msg}"),
            EmlError::NonFiniteOutput(msg) => write!(f, "non-finite output: {msg}"),
            EmlError::StackUnderflow => write!(f, "stack underflow"),
            EmlError::StackNotSingleton { len } => {
                write!(f, "stack must end with one value, got {len}")
            }
            EmlError::NonRealOutput { imag, tolerance } => {
                write!(
                    f,
                    "expected near-real output but got imag={imag} (tolerance={tolerance})"
                )
            }
            EmlError::Parse(msg) => write!(f, "parse error: {msg}"),
            EmlError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for EmlError {}

impl From<eml_core::EmlCoreError> for EmlError {
    fn from(value: eml_core::EmlCoreError) -> Self {
        match value {
            eml_core::EmlCoreError::Domain(msg) => EmlError::Domain(msg),
            eml_core::EmlCoreError::NonFiniteInput(msg) => EmlError::NonFiniteInput(msg),
            eml_core::EmlCoreError::NonFiniteOutput(msg) => EmlError::NonFiniteOutput(msg),
        }
    }
}

impl From<eml_lowering::LoweringError> for EmlError {
    fn from(value: eml_lowering::LoweringError) -> Self {
        match value {
            eml_lowering::LoweringError::Domain(msg) => EmlError::Domain(msg),
            eml_lowering::LoweringError::MissingVariable { index, arity } => {
                EmlError::MissingVariable { index, arity }
            }
            eml_lowering::LoweringError::Parse(msg) => EmlError::Parse(msg),
            eml_lowering::LoweringError::Unsupported(msg) => EmlError::Unsupported(msg),
            eml_lowering::LoweringError::Overflow(msg) => EmlError::Parse(msg.to_string()),
        }
    }
}
