//! High-level compile/evaluate pipeline.
//!
//! The lower-level modules remain available for research control. This module
//! provides a narrower API for "parse -> optimize -> lower -> compile ->
//! evaluate/verify" workflows.

use num_complex::Complex64;

use crate::bytecode::BytecodeProgram;
use crate::core::EvalPolicy;
use crate::ir::{eval_rpn_complex_with_policy, eval_rpn_real_with_policy, Expr, ExprStats, Token};
use crate::lowering::{
    lower_to_eml, parse_source_expr, raise_expr_to_source, source_expr_node_count, SourceExpr,
};
use crate::opt::optimize_for_lowering;
use crate::plugin::{
    ExecutionBackend, ExprPass, PipelineEvent, PipelineObserver, PipelineStage, SourcePass,
};
use crate::verify::{
    verify_against_complex_ref_with_policy, verify_against_real_ref_with_policy, VerificationReport,
};
use crate::{EmlError, EmlResult};

/// Builtin evaluator choices exposed by the high-level API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinBackend {
    /// Recursive tree evaluation.
    Tree,
    /// Stack-based reverse-polish execution.
    Rpn,
    /// Register bytecode execution.
    Bytecode,
}

/// Compile-time options for the high-level pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineOptions {
    /// Whether to run builtin source optimization before user passes.
    pub optimize_source: bool,
    /// Whether to precompile bytecode.
    pub compile_bytecode: bool,
    /// Evaluation policy used by bytecode compilation and default execution.
    pub eval_policy: EvalPolicy,
    /// Tolerance used by real-valued evaluation helpers.
    pub imag_tolerance: f64,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            optimize_source: true,
            compile_bytecode: true,
            eval_policy: EvalPolicy::default(),
            imag_tolerance: 1e-12,
        }
    }
}

/// Compact compile report for observability and docs examples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineReport {
    /// Source expression node count before optimization.
    pub input_source_nodes: usize,
    /// Source expression node count after builtin and user passes.
    pub optimized_source_nodes: usize,
    /// Lowered EML expression statistics.
    pub expr_stats: ExprStats,
    /// Bytecode instruction count when compiled.
    pub bytecode_instructions: Option<usize>,
    /// Whether builtin optimization was enabled.
    pub used_builtin_optimization: bool,
}

/// Fully compiled pipeline artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledPipeline {
    source: SourceExpr,
    optimized_source: SourceExpr,
    expr: Expr,
    rpn: Vec<Token>,
    bytecode: Option<BytecodeProgram>,
    report: PipelineReport,
    eval_policy: EvalPolicy,
    imag_tolerance: f64,
}

impl CompiledPipeline {
    /// Returns the original source expression.
    pub fn source(&self) -> &SourceExpr {
        &self.source
    }

    /// Returns the optimized source expression that was actually lowered.
    pub fn optimized_source(&self) -> &SourceExpr {
        &self.optimized_source
    }

    /// Returns the lowered EML IR tree.
    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns the precomputed RPN token stream.
    pub fn rpn(&self) -> &[Token] {
        &self.rpn
    }

    /// Returns the optional precompiled bytecode program.
    pub fn bytecode(&self) -> Option<&BytecodeProgram> {
        self.bytecode.as_ref()
    }

    /// Returns the compile report.
    pub fn report(&self) -> &PipelineReport {
        &self.report
    }

    /// Returns the source-level approximation obtained from the final IR.
    pub fn raised_source(&self) -> SourceExpr {
        raise_expr_to_source(&self.expr)
    }

    /// Evaluates via one of the builtin backends.
    pub fn eval_complex(
        &self,
        backend: BuiltinBackend,
        vars: &[Complex64],
    ) -> EmlResult<Complex64> {
        match backend {
            BuiltinBackend::Tree => self.expr.eval_complex_with_policy(vars, &self.eval_policy),
            BuiltinBackend::Rpn => eval_rpn_complex_with_policy(&self.rpn, vars, &self.eval_policy),
            BuiltinBackend::Bytecode => self
                .bytecode
                .as_ref()
                .ok_or(EmlError::Unsupported("bytecode backend was not compiled"))?
                .eval_complex_with_policy(vars, &self.eval_policy),
        }
    }

    /// Evaluates via one of the builtin backends over real inputs.
    pub fn eval_real(&self, backend: BuiltinBackend, vars: &[f64]) -> EmlResult<f64> {
        match backend {
            BuiltinBackend::Tree => {
                self.expr
                    .eval_real_with_policy(vars, self.imag_tolerance, &self.eval_policy)
            }
            BuiltinBackend::Rpn => {
                eval_rpn_real_with_policy(&self.rpn, vars, self.imag_tolerance, &self.eval_policy)
            }
            BuiltinBackend::Bytecode => self
                .bytecode
                .as_ref()
                .ok_or(EmlError::Unsupported("bytecode backend was not compiled"))?
                .eval_real_with_policy(vars, self.imag_tolerance, &self.eval_policy),
        }
    }

    /// Evaluates through a user-provided experimental backend.
    pub fn eval_complex_with_backend(
        &self,
        backend: &dyn ExecutionBackend,
        vars: &[Complex64],
    ) -> EmlResult<Complex64> {
        backend.eval_complex(&self.expr, vars, &self.eval_policy)
    }

    /// Verifies builtin tree evaluation against a complex reference.
    pub fn verify_against_complex_ref(
        &self,
        samples: &[Vec<Complex64>],
        tolerance: f64,
        reference: impl Fn(&[Complex64]) -> Complex64,
    ) -> VerificationReport {
        verify_against_complex_ref_with_policy(
            &self.expr,
            samples,
            tolerance,
            &self.eval_policy,
            reference,
        )
    }

    /// Verifies builtin tree evaluation against a real reference.
    pub fn verify_against_real_ref(
        &self,
        samples: &[Vec<f64>],
        tolerance: f64,
        reference: impl Fn(&[f64]) -> f64,
    ) -> VerificationReport {
        verify_against_real_ref_with_policy(
            &self.expr,
            samples,
            self.imag_tolerance,
            tolerance,
            &self.eval_policy,
            reference,
        )
    }
}

/// High-level builder with optional research-time extension hooks.
#[derive(Default)]
pub struct PipelineBuilder {
    options: PipelineOptions,
    source_passes: Vec<Box<dyn SourcePass>>,
    expr_passes: Vec<Box<dyn ExprPass>>,
    observers: Vec<Box<dyn PipelineObserver>>,
}

impl PipelineBuilder {
    /// Creates a new builder with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces compile-time options.
    pub fn with_options(mut self, options: PipelineOptions) -> Self {
        self.options = options;
        self
    }

    /// Appends a source-level pass.
    pub fn with_source_pass(mut self, pass: impl SourcePass + 'static) -> Self {
        self.source_passes.push(Box::new(pass));
        self
    }

    /// Appends an IR-level pass.
    pub fn with_expr_pass(mut self, pass: impl ExprPass + 'static) -> Self {
        self.expr_passes.push(Box::new(pass));
        self
    }

    /// Appends a compile-time observer.
    pub fn with_observer(mut self, observer: impl PipelineObserver + 'static) -> Self {
        self.observers.push(Box::new(observer));
        self
    }

    /// Parses and compiles an infix source string.
    pub fn compile_str(self, input: &str) -> EmlResult<CompiledPipeline> {
        let source = parse_source_expr(input)?;
        self.compile_source(source)
    }

    /// Compiles a prebuilt source expression.
    pub fn compile_source(self, source: SourceExpr) -> EmlResult<CompiledPipeline> {
        let PipelineBuilder {
            options,
            source_passes,
            expr_passes,
            observers,
        } = self;

        emit(
            &observers,
            PipelineEvent::from_source(PipelineStage::Parsed, None, &source),
        );

        let mut optimized_source = if options.optimize_source {
            optimize_for_lowering(&source)
        } else {
            source.clone()
        };
        emit(
            &observers,
            PipelineEvent::from_source(PipelineStage::OptimizedSource, None, &optimized_source),
        );

        for pass in &source_passes {
            optimized_source = pass.run(&optimized_source)?;
            emit(
                &observers,
                PipelineEvent::from_source(
                    PipelineStage::SourcePass,
                    Some(pass.name().to_string()),
                    &optimized_source,
                ),
            );
        }

        let mut expr = lower_to_eml(&optimized_source)?;
        emit(
            &observers,
            PipelineEvent::from_expr(PipelineStage::Lowered, None, &expr),
        );

        for pass in &expr_passes {
            expr = pass.run(&expr)?;
            emit(
                &observers,
                PipelineEvent::from_expr(
                    PipelineStage::ExprPass,
                    Some(pass.name().to_string()),
                    &expr,
                ),
            );
        }

        let rpn = expr.to_rpn_vec();
        let bytecode = if options.compile_bytecode {
            let prog = BytecodeProgram::from_expr_with_policy(&expr, &options.eval_policy)?;
            let event = PipelineEvent {
                stage: PipelineStage::BytecodeCompiled,
                label: None,
                source_nodes: None,
                expr_nodes: Some(expr.stats().nodes),
                expr_depth: Some(expr.stats().depth),
                bytecode_instructions: Some(prog.instructions.len()),
            };
            emit(&observers, event);
            Some(prog)
        } else {
            None
        };

        let expr_stats = expr.stats();
        let report = PipelineReport {
            input_source_nodes: source_expr_node_count(&source),
            optimized_source_nodes: source_expr_node_count(&optimized_source),
            expr_stats: expr_stats.clone(),
            bytecode_instructions: bytecode.as_ref().map(|prog| prog.instructions.len()),
            used_builtin_optimization: options.optimize_source,
        };

        Ok(CompiledPipeline {
            source,
            optimized_source,
            expr,
            rpn,
            bytecode,
            report,
            eval_policy: options.eval_policy,
            imag_tolerance: options.imag_tolerance,
        })
    }
}

/// Compiles an infix expression with default pipeline options.
pub fn compile(input: &str) -> EmlResult<CompiledPipeline> {
    PipelineBuilder::new().compile_str(input)
}

fn emit(observers: &[Box<dyn PipelineObserver>], event: PipelineEvent) {
    for observer in observers {
        observer.on_event(&event);
    }
}
