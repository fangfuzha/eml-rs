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
fn parallel_complex_batch_eval_matches_serial_bytecode() {
    let pipeline = PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap();
    let samples = complex_samples(256);
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial = pipeline
        .eval_complex_batch(BuiltinBackend::Bytecode, &samples)
        .unwrap();
    let parallel = pipeline
        .eval_complex_batch_parallel(BuiltinBackend::Bytecode, &samples, parallelism)
        .unwrap();

    assert_eq!(parallel, serial);
}

#[test]
fn parallel_real_batch_eval_matches_serial_bytecode() {
    let pipeline = PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap();
    let samples = real_samples(256);
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial = pipeline
        .eval_real_batch(BuiltinBackend::Bytecode, &samples)
        .unwrap();
    let parallel = pipeline
        .eval_real_batch_parallel(BuiltinBackend::Bytecode, &samples, parallelism)
        .unwrap();

    assert_eq!(parallel, serial);
}

#[test]
fn default_bytecode_batch_eval_matches_serial_program() {
    let pipeline = PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1) + exp(x2) + exp(x3)")
        .unwrap();
    let samples = (0..1_024)
        .map(|i| {
            let base = 0.01 + (i as f64) * 0.0005;
            vec![
                Complex64::new(base, 0.0),
                Complex64::new(base + 0.05, 0.0),
                Complex64::new(base + 0.1, 0.0),
                Complex64::new(base + 0.15, 0.0),
            ]
        })
        .collect::<Vec<_>>();

    let serial = pipeline
        .bytecode()
        .unwrap()
        .eval_complex_batch_with_policy(&samples, &eml_rs::core::EvalPolicy::default())
        .unwrap();
    let automatic = pipeline
        .eval_complex_batch(BuiltinBackend::Bytecode, &samples)
        .unwrap();

    assert_eq!(automatic, serial);
}
