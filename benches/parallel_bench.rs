use std::hint::black_box;
use std::thread;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use eml_rs::api::{BuiltinBackend, PipelineBuilder};
use eml_rs::bytecode::BytecodeProgram;
use eml_rs::lowering::{cross_entropy_template, lower_to_eml, SourceExpr};
use eml_rs::verify::VerifyParallelism;
use eml_rs::EmlError;
use num_complex::Complex64;

const PARALLEL_THRESHOLD_BATCH_SIZES: &[usize] = &[32, 64, 128, 256, 512, 1_024];
const BYTECODE_PARALLEL_BATCH_SIZES: &[usize] = &[256, 1_024, 4_096];

fn positive_real_samples(n: usize, arity: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            (0..arity)
                .map(|j| {
                    let x = 0.1 + (i as f64) * 0.001 + (j as f64) * 0.05;
                    Complex64::new(x, 0.0)
                })
                .collect()
        })
        .collect()
}

fn parallel_threshold_pipeline() -> eml_rs::api::CompiledPipeline {
    PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap()
}

fn build_softmax_ce_bytecode() -> BytecodeProgram {
    let logits = vec![
        SourceExpr::var(0),
        SourceExpr::var(1),
        SourceExpr::var(2),
        SourceExpr::var(3),
    ];
    let ce = cross_entropy_template(&logits, 2).unwrap();
    let expr = lower_to_eml(&ce).unwrap();
    BytecodeProgram::from_expr(&expr).unwrap()
}

fn eval_bytecode_batch_parallel(
    program: &BytecodeProgram,
    samples: &[Vec<Complex64>],
    parallelism: VerifyParallelism,
) -> Result<Vec<Complex64>, EmlError> {
    let workers = parallelism.effective_workers(samples.len());
    if workers <= 1 {
        return program.eval_complex_batch(samples);
    }

    let chunk_size = samples.len().div_ceil(workers);
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for chunk in samples.chunks(chunk_size) {
            handles.push(scope.spawn(move || program.eval_complex_batch(chunk)));
        }

        let mut out = Vec::with_capacity(samples.len());
        for handle in handles {
            let chunk = handle
                .join()
                .expect("bytecode batch worker unexpectedly panicked")?;
            out.extend(chunk);
        }
        Ok(out)
    })
}

fn bench_parallel_threshold_tree(c: &mut Criterion) {
    let pipeline = parallel_threshold_pipeline();
    let parallelism = VerifyParallelism::auto();

    for batch_size in PARALLEL_THRESHOLD_BATCH_SIZES {
        let samples = positive_real_samples(*batch_size, 2);
        let serial_name = format!("parallel_tree_serial_batch{batch_size}");
        let auto_name = format!("parallel_tree_auto_batch{batch_size}");

        c.bench_function(&serial_name, |b| {
            b.iter(|| {
                black_box(
                    pipeline
                        .eval_complex_batch(BuiltinBackend::Tree, black_box(&samples))
                        .unwrap(),
                );
            })
        });

        c.bench_function(&auto_name, |b| {
            b.iter(|| {
                black_box(
                    pipeline
                        .eval_complex_batch_parallel(
                            BuiltinBackend::Tree,
                            black_box(&samples),
                            parallelism,
                        )
                        .unwrap(),
                );
            })
        });
    }
}

fn bench_parallel_threshold_rpn(c: &mut Criterion) {
    let pipeline = parallel_threshold_pipeline();
    let parallelism = VerifyParallelism::auto();

    for batch_size in PARALLEL_THRESHOLD_BATCH_SIZES {
        let samples = positive_real_samples(*batch_size, 2);
        let serial_name = format!("parallel_rpn_serial_batch{batch_size}");
        let auto_name = format!("parallel_rpn_auto_batch{batch_size}");

        c.bench_function(&serial_name, |b| {
            b.iter(|| {
                black_box(
                    pipeline
                        .eval_complex_batch(BuiltinBackend::Rpn, black_box(&samples))
                        .unwrap(),
                );
            })
        });

        c.bench_function(&auto_name, |b| {
            b.iter(|| {
                black_box(
                    pipeline
                        .eval_complex_batch_parallel(
                            BuiltinBackend::Rpn,
                            black_box(&samples),
                            parallelism,
                        )
                        .unwrap(),
                );
            })
        });
    }
}

fn bench_bytecode_parallel_candidate(c: &mut Criterion) {
    let program = build_softmax_ce_bytecode();
    let parallelism = VerifyParallelism::auto();

    for batch_size in BYTECODE_PARALLEL_BATCH_SIZES {
        let samples = positive_real_samples(*batch_size, 4);
        let serial_name = format!("parallel_bytecode_serial_batch{batch_size}");
        let auto_name = format!("parallel_bytecode_auto_batch{batch_size}");

        c.bench_function(&serial_name, |b| {
            b.iter(|| {
                black_box(program.eval_complex_batch(black_box(&samples)).unwrap());
            })
        });

        c.bench_function(&auto_name, |b| {
            b.iter(|| {
                black_box(
                    eval_bytecode_batch_parallel(
                        black_box(&program),
                        black_box(&samples),
                        parallelism,
                    )
                    .unwrap(),
                );
            })
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(40)
        .measurement_time(Duration::from_secs(6))
        .warm_up_time(Duration::from_secs(2));
    targets =
        bench_parallel_threshold_tree,
        bench_parallel_threshold_rpn,
        bench_bytecode_parallel_candidate
}
criterion_main!(benches);
