use std::time::Duration;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use eml_rs::api::{BuiltinBackend, PipelineBuilder};
use eml_rs::bytecode::BytecodeProgram;
use eml_rs::ir::{eval_rpn_complex, Expr};
use eml_rs::lowering::{
    cross_entropy_template, eval_source_expr_complex, lower_to_eml, source_expr_node_count,
    SourceExpr,
};
use eml_rs::verify::{verify_against_complex_ref, VerifyParallelism};
use num_complex::Complex64;

const SOFTMAX_CE_BATCH_SIZES: &[usize] = &[32, 256, 1_024, 4_096];
const LOWER_VERIFY_TARGETS: &[(usize, &str)] = &[(1_000, "1k"), (10_000, "10k"), (100_000, "100k")];
const PARALLEL_THRESHOLD_BATCH_SIZES: &[usize] = &[32, 64, 128, 256, 512, 1_024];

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

fn bench_eml_ln_tree(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("eml_ln_tree_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(expr.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_eml_ln_rpn(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let tokens = expr.to_rpn_vec();
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("eml_ln_rpn_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(eval_rpn_complex(black_box(&tokens), black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_eml_ln_bytecode(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let prog = BytecodeProgram::from_expr(&expr).unwrap();
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("eml_ln_bytecode_eval", |b| {
        b.iter(|| {
            black_box(prog.eval_complex_batch(black_box(&samples)).unwrap());
        })
    });
}

fn build_shared_expr() -> Expr {
    // Repeated identical subtrees let the bytecode path pay off via CSE.
    let leaf = Expr::ln(Expr::exp(Expr::var(0)));
    let pair = Expr::eml(leaf.clone(), leaf.clone());
    let quad = Expr::eml(pair.clone(), pair.clone());
    Expr::eml(quad.clone(), quad)
}

fn bench_shared_tree(c: &mut Criterion) {
    let expr = build_shared_expr();
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("shared_eml_tree_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(expr.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_shared_bytecode(c: &mut Criterion) {
    let expr = build_shared_expr();
    let prog = BytecodeProgram::from_expr(&expr).unwrap();
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("shared_eml_bytecode_eval", |b| {
        b.iter(|| {
            black_box(prog.eval_complex_batch(black_box(&samples)).unwrap());
        })
    });
}

fn bench_native_ln(c: &mut Criterion) {
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("native_complex_ln", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(vars[0].ln());
            }
        })
    });
}

fn build_softmax_ce_expr() -> Expr {
    let logits = vec![
        SourceExpr::var(0),
        SourceExpr::var(1),
        SourceExpr::var(2),
        SourceExpr::var(3),
    ];
    let ce = cross_entropy_template(&logits, 2).unwrap();
    lower_to_eml(&ce).unwrap()
}

fn bench_softmax_ce_tree(c: &mut Criterion) {
    for batch_size in SOFTMAX_CE_BATCH_SIZES {
        bench_softmax_ce_tree_batch(c, *batch_size);
    }
}

fn bench_softmax_ce_tree_batch(c: &mut Criterion, batch_size: usize) {
    let expr = build_softmax_ce_expr();
    let samples = positive_real_samples(batch_size, 4);
    let name = format!("softmax_ce_tree_eval_batch{batch_size}");

    c.bench_function(&name, |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(expr.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_softmax_ce_bytecode(c: &mut Criterion) {
    for batch_size in SOFTMAX_CE_BATCH_SIZES {
        bench_softmax_ce_bytecode_batch(c, *batch_size);
    }
}

fn bench_softmax_ce_bytecode_batch(c: &mut Criterion, batch_size: usize) {
    let expr = build_softmax_ce_expr();
    let prog = BytecodeProgram::from_expr(&expr).unwrap();
    let samples = positive_real_samples(batch_size, 4);
    let name = format!("softmax_ce_bytecode_eval_batch{batch_size}");

    c.bench_function(&name, |b| {
        b.iter(|| {
            black_box(prog.eval_complex_batch(black_box(&samples)).unwrap());
        })
    });
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

fn bench_lower_verify_nodes(c: &mut Criterion, target_nodes: usize, label: &str) {
    let source = build_target_sized_source_expr(target_nodes);
    let samples = positive_real_samples(4, 2);
    let name = format!("lower_verify_{label}_nodes");

    c.bench_function(&name, |b| {
        b.iter(|| {
            let lowered = lower_to_eml(black_box(&source)).unwrap();
            let report = verify_against_complex_ref(
                black_box(&lowered),
                black_box(&samples),
                1e-9,
                |vars| eval_source_expr_complex(&source, vars).unwrap(),
            );
            black_box(report);
        })
    });
}

fn bench_lower_verify(c: &mut Criterion) {
    for (target_nodes, label) in LOWER_VERIFY_TARGETS {
        bench_lower_verify_nodes(c, *target_nodes, label);
    }
}

fn parallel_threshold_pipeline() -> eml_rs::api::CompiledPipeline {
    PipelineBuilder::new()
        .compile_str("exp(x0) + exp(x1)")
        .unwrap()
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

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(60)
        .measurement_time(Duration::from_secs(8))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_eml_ln_tree,
        bench_eml_ln_rpn,
        bench_eml_ln_bytecode,
        bench_shared_tree,
        bench_shared_bytecode,
        bench_native_ln,
        bench_softmax_ce_tree,
        bench_softmax_ce_bytecode,
        bench_lower_verify,
        bench_parallel_threshold_tree,
        bench_parallel_threshold_rpn
}
criterion_main!(benches);
