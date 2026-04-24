//! Lightweight compile/evaluate profiling structures for research workloads.

use std::time::Duration;

use crate::api::BuiltinBackend;

/// Stage-by-stage compile metrics collected by the high-level pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileMetrics {
    /// Time spent parsing source text. Zero for prebuilt source trees.
    pub parse: Duration,
    /// Time spent in builtin source optimization plus user source passes.
    pub simplify: Duration,
    /// Time spent lowering source AST into runtime EML IR.
    pub lowering: Duration,
    /// Time spent in user IR passes after lowering.
    pub expr_pass: Duration,
    /// Time spent building the RPN token stream.
    pub rpn_build: Duration,
    /// Time spent building register bytecode when enabled.
    pub bytecode_build: Option<Duration>,
    /// End-to-end compile time for the profiled call.
    pub total: Duration,
    /// Source node count before optimization.
    pub input_source_nodes: usize,
    /// Source node count after optimization and source passes.
    pub optimized_source_nodes: usize,
    /// Lowered IR node count.
    pub expr_nodes: usize,
    /// Lowered IR depth.
    pub expr_depth: usize,
    /// Lowered IR unique subtree count.
    pub expr_unique_subexpressions: usize,
    /// Bytecode instruction count when compiled.
    pub bytecode_instructions: Option<usize>,
}

impl Default for CompileMetrics {
    fn default() -> Self {
        Self {
            parse: Duration::ZERO,
            simplify: Duration::ZERO,
            lowering: Duration::ZERO,
            expr_pass: Duration::ZERO,
            rpn_build: Duration::ZERO,
            bytecode_build: None,
            total: Duration::ZERO,
            input_source_nodes: 0,
            optimized_source_nodes: 0,
            expr_nodes: 0,
            expr_depth: 0,
            expr_unique_subexpressions: 0,
            bytecode_instructions: None,
        }
    }
}

/// Batch evaluation metrics for a builtin backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalMetrics {
    /// Which builtin backend was timed.
    pub backend: BuiltinBackend,
    /// Number of samples evaluated.
    pub samples: usize,
    /// Total elapsed time across the batch.
    pub total: Duration,
    /// Average elapsed time per sample.
    pub per_sample: Duration,
}

/// Bundle returned by profiled compilation helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfiledPipeline<T> {
    /// Compiled artifact produced by the pipeline.
    pub pipeline: T,
    /// Compile-time metrics for the artifact.
    pub metrics: CompileMetrics,
}
