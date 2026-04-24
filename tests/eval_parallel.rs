use eml_rs::api::{BuiltinBackend, PipelineBuilder};
use eml_rs::verify::VerifyParallelism;
use num_complex::Complex64;

fn complex_samples(n: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            vec![
                Complex64::new(0.1 + (i as f64) * 0.01, 0.0),
                Complex64::new(0.3 + (i as f64) * 0.005, 0.0),
            ]
        })
        .collect()
}

fn real_samples(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| vec![0.05 + (i as f64) * 0.01, 0.2 + (i as f64) * 0.005])
        .collect()
}

#[test]
fn parallel_complex_batch_eval_matches_serial_tree() {
    let pipeline = PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap();
    let samples = complex_samples(256);
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial = pipeline
        .eval_complex_batch(BuiltinBackend::Tree, &samples)
        .unwrap();
    let parallel = pipeline
        .eval_complex_batch_parallel(BuiltinBackend::Tree, &samples, parallelism)
        .unwrap();

    assert_eq!(parallel, serial);
}

#[test]
fn parallel_real_batch_eval_matches_serial_rpn() {
    let pipeline = PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap();
    let samples = real_samples(256);
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial = pipeline
        .eval_real_batch(BuiltinBackend::Rpn, &samples)
        .unwrap();
    let parallel = pipeline
        .eval_real_batch_parallel(BuiltinBackend::Rpn, &samples, parallelism)
        .unwrap();

    assert_eq!(parallel, serial);
}

#[test]
fn parallel_batch_eval_rejects_bytecode_backend() {
    let pipeline = PipelineBuilder::new().compile_str("exp(x0)").unwrap();
    let samples = complex_samples(8);

    let err = pipeline
        .eval_complex_batch_parallel(
            BuiltinBackend::Bytecode,
            &samples,
            VerifyParallelism::auto(),
        )
        .unwrap_err();

    assert!(
        err.to_string().contains("Tree/Rpn"),
        "unexpected error: {err}"
    );
}
