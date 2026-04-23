//! Extension points for research-time customization.
//!
//! `eml-rs` keeps the runtime IR intentionally tiny, so plugin support is
//! centered on:
//! - source-to-source passes before lowering,
//! - IR-to-IR passes after lowering,
//! - observers for instrumentation,
//! - optional custom execution backends.

use num_complex::Complex64;

use crate::core::EvalPolicy;
use crate::ir::Expr;
use crate::lowering::{source_expr_node_count, SourceExpr};
use crate::EmlResult;

/// Source-level pass executed before lowering to EML IR.
pub trait SourcePass: Send + Sync {
    /// Stable display name used by reports and observers.
    fn name(&self) -> &'static str;

    /// Transforms a source expression.
    fn run(&self, expr: &SourceExpr) -> EmlResult<SourceExpr>;
}

/// IR-level pass executed after lowering to pure EML.
pub trait ExprPass: Send + Sync {
    /// Stable display name used by reports and observers.
    fn name(&self) -> &'static str;

    /// Transforms an EML IR tree.
    fn run(&self, expr: &Expr) -> EmlResult<Expr>;
}

/// Optional custom executor for experiments.
pub trait ExecutionBackend: Send + Sync {
    /// Stable backend name for logs and reports.
    fn name(&self) -> &'static str;

    /// Evaluates a pure EML expression under a chosen policy.
    fn eval_complex(
        &self,
        expr: &Expr,
        vars: &[Complex64],
        policy: &EvalPolicy,
    ) -> EmlResult<Complex64>;
}

/// Observer hook used by the high-level pipeline.
pub trait PipelineObserver: Send + Sync {
    /// Receives stage notifications during compile-time orchestration.
    fn on_event(&self, event: &PipelineEvent);
}

/// Pipeline stage kind emitted to observers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Initial parsed or directly provided source expression.
    Parsed,
    /// Builtin source optimizer finished.
    OptimizedSource,
    /// A user source pass finished.
    SourcePass,
    /// Lowering to pure EML finished.
    Lowered,
    /// A user IR pass finished.
    ExprPass,
    /// Bytecode compilation finished.
    BytecodeCompiled,
}

/// Lightweight event emitted by the high-level pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineEvent {
    /// Stage kind.
    pub stage: PipelineStage,
    /// Optional label, typically the pass name.
    pub label: Option<String>,
    /// Source-tree node count when available.
    pub source_nodes: Option<usize>,
    /// EML-tree node count when available.
    pub expr_nodes: Option<usize>,
    /// EML-tree depth when available.
    pub expr_depth: Option<usize>,
    /// Bytecode instruction count when available.
    pub bytecode_instructions: Option<usize>,
}

impl PipelineEvent {
    /// Convenience constructor for source-stage events.
    pub fn from_source(stage: PipelineStage, label: Option<String>, expr: &SourceExpr) -> Self {
        Self {
            stage,
            label,
            source_nodes: Some(source_expr_node_count(expr)),
            expr_nodes: None,
            expr_depth: None,
            bytecode_instructions: None,
        }
    }

    /// Convenience constructor for IR-stage events.
    pub fn from_expr(stage: PipelineStage, label: Option<String>, expr: &Expr) -> Self {
        let stats = expr.stats();
        Self {
            stage,
            label,
            source_nodes: None,
            expr_nodes: Some(stats.nodes),
            expr_depth: Some(stats.depth),
            bytecode_instructions: None,
        }
    }
}
