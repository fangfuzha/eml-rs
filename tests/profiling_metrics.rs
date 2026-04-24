use eml_rs::api::{BuiltinBackend, PipelineBuilder, PipelineOptions};
use num_complex::Complex64;

#[test]
fn profiled_compile_records_stage_metrics() {
    let profiled = PipelineBuilder::new()
        .compile_str_profiled("softplus(x0) + mish(x0) + log(x1 + 2)")
        .unwrap();

    assert!(profiled.metrics.input_source_nodes > 0);
    assert!(profiled.metrics.optimized_source_nodes > 0);
    assert!(profiled.metrics.expr_nodes > 0);
    assert!(profiled.metrics.expr_depth > 0);
    assert!(profiled.metrics.bytecode_build.is_some());
    assert_eq!(
        profiled.metrics.bytecode_instructions,
        profiled.pipeline.report().bytecode_instructions
    );
}

#[test]
fn profiled_eval_records_backend_and_sample_count() {
    let profiled = PipelineBuilder::new()
        .compile_str_profiled("exp(x0)")
        .unwrap();
    let samples = vec![
        vec![Complex64::new(0.2, 0.0)],
        vec![Complex64::new(0.4, 0.0)],
        vec![Complex64::new(0.6, 0.0)],
    ];

    let metrics = profiled
        .pipeline
        .profile_eval_complex_batch(BuiltinBackend::Bytecode, &samples)
        .unwrap();
    assert_eq!(metrics.backend, BuiltinBackend::Bytecode);
    assert_eq!(metrics.samples, samples.len());
}

#[test]
fn profiled_compile_without_bytecode_reports_none() {
    let options = PipelineOptions {
        compile_bytecode: false,
        ..PipelineOptions::default()
    };
    let profiled = PipelineBuilder::new()
        .with_options(options)
        .compile_str_profiled("exp(x0) - log(x1 + 2)")
        .unwrap();

    assert!(profiled.metrics.bytecode_build.is_none());
    assert!(profiled.metrics.bytecode_instructions.is_none());
}
