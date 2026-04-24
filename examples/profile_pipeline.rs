use eml_rs::api::{BuiltinBackend, PipelineBuilder, PipelineOptions};
use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{batch_cross_entropy_mean_template, source_expr_node_count, SourceExpr};
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
) {
    for backend in [
        BuiltinBackend::Tree,
        BuiltinBackend::Rpn,
        BuiltinBackend::Bytecode,
    ] {
        if backend == BuiltinBackend::Bytecode && profiled.pipeline.bytecode().is_none() {
            continue;
        }
        let metrics = profiled
            .pipeline
            .profile_eval_complex_batch(backend, samples)
            .unwrap();
        println!(
            "{name} eval {:?}: total={:?}, per_sample={:?}, samples={}",
            metrics.backend, metrics.total, metrics.per_sample, metrics.samples
        );
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

fn main() {
    let options = PipelineOptions {
        eval_policy: EvalPolicy::relaxed(),
        ..PipelineOptions::default()
    };
    let lower_10k = build_target_sized_source_expr(10_000);
    let lower_profile = PipelineBuilder::new()
        .with_options(options.clone())
        .compile_source_profiled(lower_10k)
        .unwrap();
    print_compile_metrics("lower_10k", &lower_profile);
    print_backend_metrics("lower_10k", &lower_profile, &positive_real_samples(256, 2));

    let softmax_ce = build_softmax_ce_mean_expr();
    let softmax_profile = PipelineBuilder::new()
        .with_options(options)
        .compile_source_profiled(softmax_ce)
        .unwrap();
    print_compile_metrics("softmax_ce_mean", &softmax_profile);
    print_backend_metrics(
        "softmax_ce_mean",
        &softmax_profile,
        &positive_real_samples(1024, 4),
    );
}
