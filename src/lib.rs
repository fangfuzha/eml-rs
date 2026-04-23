use std::fmt::{Display, Formatter};

pub mod core;
pub mod ir;
pub mod verify;

#[derive(Debug, Clone, PartialEq)]
pub enum EmlError {
    Domain(&'static str),
    MissingVariable { index: usize, arity: usize },
    NonFiniteInput(&'static str),
    NonFiniteOutput(&'static str),
    StackUnderflow,
    StackNotSingleton { len: usize },
    NonRealOutput { imag: f64, tolerance: f64 },
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
            EmlError::StackUnderflow => write!(f, "RPN stack underflow"),
            EmlError::StackNotSingleton { len } => {
                write!(f, "RPN stack must end with one value, got {len}")
            }
            EmlError::NonRealOutput { imag, tolerance } => {
                write!(
                    f,
                    "expected near-real output but got imag={imag} (tolerance={tolerance})"
                )
            }
        }
    }
}

impl std::error::Error for EmlError {}
