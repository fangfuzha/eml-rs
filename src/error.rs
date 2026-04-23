//! Unified error types and diagnostics for `eml-rs`.

use std::fmt::{Display, Formatter};

/// Stable diagnostic code for Rust-side API errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EmlErrorCode {
    /// Mathematical domain error.
    Domain = 1000,
    /// Variable index is out of bounds.
    MissingVariable = 1001,
    /// Strict policy rejected a non-finite input.
    NonFiniteInput = 1002,
    /// Strict policy rejected a non-finite output.
    NonFiniteOutput = 1003,
    /// Stack-based evaluator underflow.
    StackUnderflow = 2000,
    /// Stack-based evaluator did not end with a singleton.
    StackNotSingleton = 2001,
    /// A real-valued path produced too much imaginary residue.
    NonRealOutput = 2002,
    /// Parsing failed.
    Parse = 3000,
    /// Requested feature is intentionally unsupported.
    Unsupported = 3001,
    /// Transformation overflowed its representable range.
    Overflow = 3002,
}

impl EmlErrorCode {
    /// Returns the numeric representation used in diagnostics.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Returns a stable symbolic name for logs and test assertions.
    pub const fn name(self) -> &'static str {
        match self {
            EmlErrorCode::Domain => "DOMAIN",
            EmlErrorCode::MissingVariable => "MISSING_VARIABLE",
            EmlErrorCode::NonFiniteInput => "NON_FINITE_INPUT",
            EmlErrorCode::NonFiniteOutput => "NON_FINITE_OUTPUT",
            EmlErrorCode::StackUnderflow => "STACK_UNDERFLOW",
            EmlErrorCode::StackNotSingleton => "STACK_NOT_SINGLETON",
            EmlErrorCode::NonRealOutput => "NON_REAL_OUTPUT",
            EmlErrorCode::Parse => "PARSE",
            EmlErrorCode::Unsupported => "UNSUPPORTED",
            EmlErrorCode::Overflow => "OVERFLOW",
        }
    }

    /// Returns a coarse diagnostic category.
    pub const fn category(self) -> &'static str {
        match self {
            EmlErrorCode::Domain
            | EmlErrorCode::MissingVariable
            | EmlErrorCode::NonFiniteInput
            | EmlErrorCode::NonFiniteOutput
            | EmlErrorCode::NonRealOutput => "semantic",
            EmlErrorCode::StackUnderflow | EmlErrorCode::StackNotSingleton => "execution",
            EmlErrorCode::Parse | EmlErrorCode::Unsupported | EmlErrorCode::Overflow => "compile",
        }
    }
}

impl Display for EmlErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.as_u16())
    }
}

/// Serializable diagnostic payload for logs, APIs, and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmlDiagnostic {
    /// Stable error code.
    pub code: EmlErrorCode,
    /// Coarse category.
    pub category: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

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
    /// Integer/rational rewrite or lowering overflowed.
    Overflow(&'static str),
}

impl EmlError {
    /// Returns the stable diagnostic code for this error.
    pub const fn code(&self) -> EmlErrorCode {
        match self {
            EmlError::Domain(_) => EmlErrorCode::Domain,
            EmlError::MissingVariable { .. } => EmlErrorCode::MissingVariable,
            EmlError::NonFiniteInput(_) => EmlErrorCode::NonFiniteInput,
            EmlError::NonFiniteOutput(_) => EmlErrorCode::NonFiniteOutput,
            EmlError::StackUnderflow => EmlErrorCode::StackUnderflow,
            EmlError::StackNotSingleton { .. } => EmlErrorCode::StackNotSingleton,
            EmlError::NonRealOutput { .. } => EmlErrorCode::NonRealOutput,
            EmlError::Parse(_) => EmlErrorCode::Parse,
            EmlError::Unsupported(_) => EmlErrorCode::Unsupported,
            EmlError::Overflow(_) => EmlErrorCode::Overflow,
        }
    }

    /// Returns the coarse category for this error.
    pub const fn category(&self) -> &'static str {
        self.code().category()
    }

    /// Returns a structured diagnostic payload.
    pub fn diagnostic(&self) -> EmlDiagnostic {
        EmlDiagnostic {
            code: self.code(),
            category: self.category(),
            message: self.to_string(),
        }
    }
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
            EmlError::Overflow(msg) => write!(f, "overflow: {msg}"),
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
            eml_lowering::LoweringError::Overflow(msg) => EmlError::Overflow(msg),
        }
    }
}

/// Common result type used by high-level APIs and plugin hooks.
pub type EmlResult<T> = Result<T, EmlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_exposes_code_and_message() {
        let err = EmlError::MissingVariable { index: 2, arity: 1 };
        let diag = err.diagnostic();
        assert_eq!(diag.code, EmlErrorCode::MissingVariable);
        assert_eq!(diag.category, "semantic");
        assert!(diag.message.contains("index 2"));
    }
}
