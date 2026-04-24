//! High-level compile/evaluate pipeline.
//!
//! The lower-level modules remain available for research control. This module
//! provides a narrower API for "parse -> optimize -> lower -> compile ->
//! evaluate/verify" workflows.

use std::thread;
use std::time::Instant;

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
use crate::profiling::{CompileMetrics, EvalMetrics, ProfiledPipeline, VerifyMetrics};
use crate::verify::{
    verify_against_complex_ref_parallel_with_policy, verify_against_complex_ref_with_policy,
    verify_against_real_ref_parallel_with_policy, verify_against_real_ref_with_policy,
    VerificationReport, VerifyParallelism,
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
    fn parallel_eval_supported(backend: BuiltinBackend) -> EmlResult<()> {
        match backend {
            BuiltinBackend::Tree | BuiltinBackend::Rpn => Ok(()),
            BuiltinBackend::Bytecode => Err(EmlError::Unsupported(
                "parallel batch evaluation currently supports only Tree/Rpn backends",
            )),
        }
    }

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

    /// Evaluates a batch of complex samples via one builtin backend.
    pub fn eval_complex_batch(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<Complex64>],
    ) -> EmlResult<Vec<Complex64>> {
        samples
            .iter()
            .map(|vars| self.eval_complex(backend, vars))
            .collect()
    }

    /// Evaluates a batch of real samples via one builtin backend.
    pub fn eval_real_batch(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<f64>],
    ) -> EmlResult<Vec<f64>> {
        samples
            .iter()
            .map(|vars| self.eval_real(backend, vars))
            .collect()
    }

    /// Evaluates complex samples in parallel across independent chunks.
    ///
    /// Only `Tree` and `Rpn` are supported in parallel mode for now.
    pub fn eval_complex_batch_parallel(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<Complex64>],
        parallelism: VerifyParallelism,
    ) -> EmlResult<Vec<Complex64>> {
        Self::parallel_eval_supported(backend)?;
        let workers = parallelism.effective_workers(samples.len());
        if workers <= 1 {
            return self.eval_complex_batch(backend, samples);
        }

        let chunk_size = samples.len().div_ceil(workers);
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(workers);
            for chunk in samples.chunks(chunk_size) {
                handles.push(scope.spawn(move || -> EmlResult<Vec<Complex64>> {
                    chunk
                        .iter()
                        .map(|vars| self.eval_complex(backend, vars))
                        .collect()
                }));
            }

            let mut out = Vec::with_capacity(samples.len());
            for handle in handles {
                let chunk = handle
                    .join()
                    .expect("complex batch worker unexpectedly panicked")?;
                out.extend(chunk);
            }
            Ok(out)
        })
    }

    /// Evaluates real samples in parallel across independent chunks.
    ///
    /// Only `Tree` and `Rpn` are supported in parallel mode for now.
    pub fn eval_real_batch_parallel(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<f64>],
        parallelism: VerifyParallelism,
    ) -> EmlResult<Vec<f64>> {
        Self::parallel_eval_supported(backend)?;
        let workers = parallelism.effective_workers(samples.len());
        if workers <= 1 {
            return self.eval_real_batch(backend, samples);
        }

        let chunk_size = samples.len().div_ceil(workers);
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(workers);
            for chunk in samples.chunks(chunk_size) {
                handles.push(scope.spawn(move || -> EmlResult<Vec<f64>> {
                    chunk
                        .iter()
                        .map(|vars| self.eval_real(backend, vars))
                        .collect()
                }));
            }

            let mut out = Vec::with_capacity(samples.len());
            for handle in handles {
                let chunk = handle
                    .join()
                    .expect("real batch worker unexpectedly panicked")?;
                out.extend(chunk);
            }
            Ok(out)
        })
    }

    /// Measures builtin complex evaluation over a batch of samples.
    pub fn profile_eval_complex_batch(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<Complex64>],
    ) -> EmlResult<EvalMetrics> {
        let started = Instant::now();
        let _ = self.eval_complex_batch(backend, samples)?;
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        Ok(EvalMetrics {
            backend,
            samples: samples.len(),
            total,
            per_sample,
            parallel: false,
            workers: 1,
        })
    }

    /// Measures builtin real-valued evaluation over a batch of samples.
    pub fn profile_eval_real_batch(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<f64>],
    ) -> EmlResult<EvalMetrics> {
        let started = Instant::now();
        let _ = self.eval_real_batch(backend, samples)?;
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        Ok(EvalMetrics {
            backend,
            samples: samples.len(),
            total,
            per_sample,
            parallel: false,
            workers: 1,
        })
    }

    /// Measures parallel complex batch evaluation over independent samples.
    pub fn profile_eval_complex_batch_parallel(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<Complex64>],
        parallelism: VerifyParallelism,
    ) -> EmlResult<EvalMetrics> {
        let workers = parallelism.effective_workers(samples.len());
        let started = Instant::now();
        let _ = self.eval_complex_batch_parallel(backend, samples, parallelism)?;
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        Ok(EvalMetrics {
            backend,
            samples: samples.len(),
            total,
            per_sample,
            parallel: workers > 1,
            workers,
        })
    }

    /// Measures parallel real-valued batch evaluation over independent samples.
    pub fn profile_eval_real_batch_parallel(
        &self,
        backend: BuiltinBackend,
        samples: &[Vec<f64>],
        parallelism: VerifyParallelism,
    ) -> EmlResult<EvalMetrics> {
        let workers = parallelism.effective_workers(samples.len());
        let started = Instant::now();
        let _ = self.eval_real_batch_parallel(backend, samples, parallelism)?;
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        Ok(EvalMetrics {
            backend,
            samples: samples.len(),
            total,
            per_sample,
            parallel: workers > 1,
            workers,
        })
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

    /// Verifies builtin tree evaluation in parallel against a complex reference.
    pub fn verify_against_complex_ref_parallel(
        &self,
        samples: &[Vec<Complex64>],
        tolerance: f64,
        parallelism: VerifyParallelism,
        reference: impl Fn(&[Complex64]) -> Complex64 + Sync,
    ) -> VerificationReport {
        verify_against_complex_ref_parallel_with_policy(
            &self.expr,
            samples,
            tolerance,
            &self.eval_policy,
            parallelism,
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

    /// Verifies builtin tree evaluation in parallel against a real reference.
    pub fn verify_against_real_ref_parallel(
        &self,
        samples: &[Vec<f64>],
        tolerance: f64,
        parallelism: VerifyParallelism,
        reference: impl Fn(&[f64]) -> f64 + Sync,
    ) -> VerificationReport {
        verify_against_real_ref_parallel_with_policy(
            &self.expr,
            samples,
            self.imag_tolerance,
            tolerance,
            &self.eval_policy,
            parallelism,
            reference,
        )
    }

    /// Measures serial complex verification against a reference function.
    pub fn profile_verify_against_complex_ref(
        &self,
        samples: &[Vec<Complex64>],
        tolerance: f64,
        reference: impl Fn(&[Complex64]) -> Complex64,
    ) -> VerifyMetrics {
        let started = Instant::now();
        let report = self.verify_against_complex_ref(samples, tolerance, reference);
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        VerifyMetrics {
            samples: samples.len(),
            total,
            per_sample,
            parallel: false,
            workers: 1,
            report,
        }
    }

    /// Measures parallel complex verification against a reference function.
    pub fn profile_verify_against_complex_ref_parallel(
        &self,
        samples: &[Vec<Complex64>],
        tolerance: f64,
        parallelism: VerifyParallelism,
        reference: impl Fn(&[Complex64]) -> Complex64 + Sync,
    ) -> VerifyMetrics {
        let workers = parallelism.effective_workers(samples.len());
        let started = Instant::now();
        let report =
            self.verify_against_complex_ref_parallel(samples, tolerance, parallelism, reference);
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        VerifyMetrics {
            samples: samples.len(),
            total,
            per_sample,
            parallel: workers > 1,
            workers,
            report,
        }
    }

    /// Measures serial real-valued verification against a reference function.
    pub fn profile_verify_against_real_ref(
        &self,
        samples: &[Vec<f64>],
        tolerance: f64,
        reference: impl Fn(&[f64]) -> f64,
    ) -> VerifyMetrics {
        let started = Instant::now();
        let report = self.verify_against_real_ref(samples, tolerance, reference);
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        VerifyMetrics {
            samples: samples.len(),
            total,
            per_sample,
            parallel: false,
            workers: 1,
            report,
        }
    }

    /// Measures parallel real-valued verification against a reference function.
    pub fn profile_verify_against_real_ref_parallel(
        &self,
        samples: &[Vec<f64>],
        tolerance: f64,
        parallelism: VerifyParallelism,
        reference: impl Fn(&[f64]) -> f64 + Sync,
    ) -> VerifyMetrics {
        let workers = parallelism.effective_workers(samples.len());
        let started = Instant::now();
        let report =
            self.verify_against_real_ref_parallel(samples, tolerance, parallelism, reference);
        let total = started.elapsed();
        let per_sample = if samples.is_empty() {
            total
        } else {
            total.div_f64(samples.len() as f64)
        };
        VerifyMetrics {
            samples: samples.len(),
            total,
            per_sample,
            parallel: workers > 1,
            workers,
            report,
        }
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
        Ok(self.compile_str_profiled(input)?.pipeline)
    }

    /// Parses and compiles an infix source string while collecting stage timings.
    pub fn compile_str_profiled(
        self,
        input: &str,
    ) -> EmlResult<ProfiledPipeline<CompiledPipeline>> {
        let total_started = Instant::now();
        let parse_started = Instant::now();
        let source = parse_source_expr(input)?;
        let mut profiled = self.compile_source_profiled(source)?;
        profiled.metrics.parse = parse_started.elapsed();
        profiled.metrics.total = total_started.elapsed();
        Ok(profiled)
    }

    /// Compiles a prebuilt source expression.
    pub fn compile_source(self, source: SourceExpr) -> EmlResult<CompiledPipeline> {
        Ok(self.compile_source_profiled(source)?.pipeline)
    }

    /// Compiles a prebuilt source expression while collecting stage timings.
    pub fn compile_source_profiled(
        self,
        source: SourceExpr,
    ) -> EmlResult<ProfiledPipeline<CompiledPipeline>> {
        let PipelineBuilder {
            options,
            source_passes,
            expr_passes,
            observers,
        } = self;
        let total_started = Instant::now();

        emit(
            &observers,
            PipelineEvent::from_source(PipelineStage::Parsed, None, &source),
        );

        let simplify_started = Instant::now();
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
        let simplify_duration = simplify_started.elapsed();

        let lowering_started = Instant::now();
        let mut expr = lower_to_eml(&optimized_source)?;
        emit(
            &observers,
            PipelineEvent::from_expr(PipelineStage::Lowered, None, &expr),
        );
        let lowering_duration = lowering_started.elapsed();

        let expr_pass_started = Instant::now();
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
        let expr_pass_duration = expr_pass_started.elapsed();

        let rpn_started = Instant::now();
        let rpn = expr.to_rpn_vec();
        let rpn_duration = rpn_started.elapsed();
        let expr_stats = expr.stats();
        let input_source_nodes = source_expr_node_count(&source);
        let optimized_source_nodes = source_expr_node_count(&optimized_source);

        let bytecode_started = Instant::now();
        let bytecode = if options.compile_bytecode {
            let prog = BytecodeProgram::from_expr_with_policy(&expr, &options.eval_policy)?;
            let event = PipelineEvent {
                stage: PipelineStage::BytecodeCompiled,
                label: None,
                source_nodes: None,
                expr_nodes: Some(expr_stats.nodes),
                expr_depth: Some(expr_stats.depth),
                bytecode_instructions: Some(prog.instructions.len()),
            };
            emit(&observers, event);
            Some(prog)
        } else {
            None
        };
        let bytecode_duration = if options.compile_bytecode {
            Some(bytecode_started.elapsed())
        } else {
            None
        };

        let report = PipelineReport {
            input_source_nodes,
            optimized_source_nodes,
            expr_stats: expr_stats.clone(),
            bytecode_instructions: bytecode.as_ref().map(|prog| prog.instructions.len()),
            used_builtin_optimization: options.optimize_source,
        };

        let pipeline = CompiledPipeline {
            source,
            optimized_source,
            expr,
            rpn,
            bytecode,
            report,
            eval_policy: options.eval_policy,
            imag_tolerance: options.imag_tolerance,
        };

        let metrics = CompileMetrics {
            simplify: simplify_duration,
            lowering: lowering_duration,
            expr_pass: expr_pass_duration,
            rpn_build: rpn_duration,
            bytecode_build: bytecode_duration,
            total: total_started.elapsed(),
            input_source_nodes: pipeline.report.input_source_nodes,
            optimized_source_nodes: pipeline.report.optimized_source_nodes,
            expr_nodes: pipeline.report.expr_stats.nodes,
            expr_depth: pipeline.report.expr_stats.depth,
            expr_unique_subexpressions: pipeline.report.expr_stats.unique_subexpressions,
            bytecode_instructions: pipeline.report.bytecode_instructions,
            ..CompileMetrics::default()
        };

        Ok(ProfiledPipeline { pipeline, metrics })
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
