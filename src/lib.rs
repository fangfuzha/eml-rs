#![deny(unsafe_op_in_unsafe_fn)]
//! eml-rs: Engineering skeleton for EML-based expression execution.
//!
//! The crate is organized into:
//! - [`api`]: high-level compile/evaluate pipeline.
//! - [`core`]: numeric EML primitives and evaluation policy.
//! - [`ir`]: tree IR, RPN conversion/eval, and IR statistics.
//! - [`bytecode`]: register bytecode compiler/executor with CSE+const-fold.
//! - [`error`]: unified Rust-side error codes and diagnostics.
//! - [`lowering`]: compatibility wrapper for the standalone parser/lowering crate.
//! - [`opt`]: rewrite rules and cost model utilities.
//! - [`plugin`]: extension points for research-time passes/backends/observers.
//! - [`verify`]: numeric cross-check helpers.
//! - [`ffi`]: C ABI exports for embedding.

pub mod api;
pub mod bytecode;
pub mod core;
pub mod error;
pub mod ffi;
pub mod ir;
pub mod lowering;
pub mod opt;
pub mod plugin;
pub mod verify;

pub use error::{EmlDiagnostic, EmlError, EmlErrorCode, EmlResult};
