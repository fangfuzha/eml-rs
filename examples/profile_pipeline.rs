use eml_rs::api::{BuiltinBackend, PipelineBuilder, PipelineOptions};
use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{
    batch_cross_entropy_mean_template, eval_source_expr_complex, source_expr_node_count, SourceExpr,
};
use eml_rs::verify::VerifyParallelism;
use num_complex::Complex64;

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

fn centered_log_safe_samples(n: usize, arity: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            (0..arity)
                .map(|j| {
                    let centered = (((i + j) % 17) as f64 - 8.0) * 0.004;
                    Complex64::new(-1.0 + centered, 0.0)
                })
                .collect()
        })
        .collect()
}

fn source_log_leaf(var_index: usize) -> SourceExpr {
    SourceExpr::Log(Box::new(SourceExpr::Add(
        Box::new(SourceExpr::Var(var_index)),
        Box::new(SourceExpr::Int(2)),
    )))
}

fn build_balanced_add_tree(mut nodes: Vec<SourceExpr>) -> SourceExpr {
    while nodes.len() > 1 {
        let mut next = Vec::with_capacity(nodes.len().div_ceil(2));
        let mut iter = nodes.into_iter();
        while let Some(lhs) = iter.next() {
            if let Some(rhs) = iter.next() {
                next.push(SourceExpr::Add(Box::new(lhs), Box::new(rhs)));
            } else {
                next.push(lhs);
            }
        }
        nodes = next;
    }
    nodes
        .pop()
        .expect("balanced tree requires at least one node")
}

fn build_target_sized_source_expr(target_nodes: usize) -> SourceExpr {
    let mut leaf_count = ((target_nodes + 1) / 5).max(1);
    loop {
        let leaves: Vec<SourceExpr> = (0..leaf_count).map(|i| source_log_leaf(i % 2)).collect();
        let expr = build_balanced_add_tree(leaves);
        if source_expr_node_count(&expr) >= target_nodes {
            return expr;
        }
        leaf_count += 1;
    }
}

fn build_softmax_ce_mean_expr() -> SourceExpr {
    let batch_logits = vec![
        vec![
            SourceExpr::var(0),
            SourceExpr::Add(Box::new(SourceExpr::var(1)), Box::new(SourceExpr::Int(1))),
            SourceExpr::Sub(Box::new(SourceExpr::var(2)), Box::new(SourceExpr::Int(1))),
            SourceExpr::var(3),
        ];
        32
    ];
    let targets = vec![2usize; 32];
    batch_cross_entropy_mean_template(&batch_logits, &targets).unwrap()
}

fn print_backend_metrics(
    name: &str,
    profiled: &eml_rs::profiling::ProfiledPipeline<eml_rs::api::CompiledPipeline>,
    samples: &[Vec<Complex64>],
    parallelism: VerifyParallelism,
) {
    for backend in [
        BuiltinBackend::Tree,
        BuiltinBackend::Rpn,
        BuiltinBackend::Bytecode,
    ] {
        if backend == BuiltinBackend::Bytecode && profiled.pipeline.bytecode().is_none() {
            continue;
        }
        let serial = profiled
            .pipeline
            .profile_eval_complex_batch(backend, samples)
            .unwrap();
        if matches!(backend, BuiltinBackend::Tree | BuiltinBackend::Rpn) {
            let parallel = profiled
                .pipeline
                .profile_eval_complex_batch_parallel(backend, samples, parallelism)
                .unwrap();
            let speedup = if parallel.total.is_zero() {
                1.0
            } else {
                serial.total.as_secs_f64() / parallel.total.as_secs_f64()
            };
            println!(
                "{name} eval {:?}: serial_total={:?}, serial_per_sample={:?}, parallel_total={:?}, parallel_per_sample={:?}, workers={}, speedup={speedup:.2}x, samples={}",
                serial.backend,
                serial.total,
                serial.per_sample,
                parallel.total,
                parallel.per_sample,
                parallel.workers,
                serial.samples
            );
        } else {
            println!(
                "{name} eval {:?}: total={:?}, per_sample={:?}, samples={}",
                serial.backend, serial.total, serial.per_sample, serial.samples
            );
        }
    }
}

fn print_compile_metrics(
    name: &str,
    profiled: &eml_rs::profiling::ProfiledPipeline<eml_rs::api::CompiledPipeline>,
) {
    let m = &profiled.metrics;
    println!(
        "{name} compile: parse={:?}, simplify={:?}, lowering={:?}, expr_pass={:?}, rpn={:?}, bytecode={:?}, total={:?}, src_nodes={} -> {}, expr_nodes={}, depth={}, unique_subexpr={}, bytecode_instr={:?}",
        m.parse,
        m.simplify,
        m.lowering,
        m.expr_pass,
        m.rpn_build,
        m.bytecode_build,
        m.total,
        m.input_source_nodes,
        m.optimized_source_nodes,
        m.expr_nodes,
        m.expr_depth,
        m.expr_unique_subexpressions,
        m.bytecode_instructions
    );
}

fn print_verify_metrics(
    name: &str,
    profiled: &eml_rs::profiling::ProfiledPipeline<eml_rs::api::CompiledPipeline>,
    source: &SourceExpr,
    samples: &[Vec<Complex64>],
    parallelism: VerifyParallelism,
) {
    let tolerance = 1e-8;
    let serial = profiled
        .pipeline
        .profile_verify_against_complex_ref(samples, tolerance, |vars| {
            eval_source_expr_complex(source, vars).unwrap()
        });
    let parallel = profiled
        .pipeline
        .profile_verify_against_complex_ref_parallel(samples, tolerance, parallelism, |vars| {
            eval_source_expr_complex(source, vars).unwrap()
        });
    let speedup = if parallel.total.is_zero() {
        1.0
    } else {
        serial.total.as_secs_f64() / parallel.total.as_secs_f64()
    };
    println!(
        "{name} verify: serial_total={:?}, serial_per_sample={:?}, parallel_total={:?}, parallel_per_sample={:?}, workers={}, speedup={speedup:.2}x, max_abs_error={:.3e}, passed={}/{}",
        serial.total,
        serial.per_sample,
        parallel.total,
        parallel.per_sample,
        parallel.workers,
        parallel.report.max_abs_error,
        parallel.report.passed,
        parallel.report.total
    );
}

fn main() {
    let options = PipelineOptions {
        eval_policy: EvalPolicy::relaxed(),
        ..PipelineOptions::default()
    };
    let verify_parallelism = VerifyParallelism {
        workers: 8,
        min_samples_per_worker: 128,
    };
    let eval_parallelism = VerifyParallelism {
        workers: 8,
        min_samples_per_worker: 32,
    };
    let lower_10k = build_target_sized_source_expr(10_000);
    let lower_profile = PipelineBuilder::new()
        .with_options(options.clone())
        .compile_source_profiled(lower_10k.clone())
        .unwrap();
    print_compile_metrics("lower_10k", &lower_profile);
    print_backend_metrics(
        "lower_10k",
        &lower_profile,
        &centered_log_safe_samples(256, 2),
        eval_parallelism,
    );
    print_verify_metrics(
        "lower_10k",
        &lower_profile,
        &lower_10k,
        &centered_log_safe_samples(2048, 2),
        verify_parallelism,
    );

    let softmax_ce = build_softmax_ce_mean_expr();
    let softmax_profile = PipelineBuilder::new()
        .with_options(options)
        .compile_source_profiled(softmax_ce.clone())
        .unwrap();
    print_compile_metrics("softmax_ce_mean", &softmax_profile);
    print_backend_metrics(
        "softmax_ce_mean",
        &softmax_profile,
        &positive_real_samples(1024, 4),
        eval_parallelism,
    );
    print_verify_metrics(
        "softmax_ce_mean",
        &softmax_profile,
        &softmax_ce,
        &positive_real_samples(2048, 4),
        verify_parallelism,
    );
}
