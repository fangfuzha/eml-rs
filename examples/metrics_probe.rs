use std::env;
use std::process;
use std::time::Instant;

use eml_rs::api::{PipelineBuilder, PipelineOptions};
use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{source_expr_node_count, SourceExpr};

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
    nodes.pop().unwrap_or(SourceExpr::Int(0))
}

fn build_target_sized_source_expr(target_nodes: usize) -> SourceExpr {
    let mut leaf_count = ((target_nodes + 5) / 5).max(1);
    loop {
        let leaves: Vec<SourceExpr> = (0..leaf_count).map(|i| source_log_leaf(i % 2)).collect();
        let expr = build_balanced_add_tree(leaves);
        if source_expr_node_count(&expr) >= target_nodes {
            return expr;
        }
        leaf_count += 1;
    }
}

fn parse_usize_arg(args: &[String], name: &str, default: usize) -> usize {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .and_then(|pair| pair[1].parse::<usize>().ok())
        .unwrap_or(default)
}

fn print_help() {
    println!(
        "metrics_probe --nodes <N> --samples <N>\n\
         构造指定规模的 SourceExpr，执行 profiled compile，并输出 JSON。"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return;
    }

    let target_nodes = parse_usize_arg(&args, "--nodes", 10_000);
    let samples = parse_usize_arg(&args, "--samples", 8);
    if target_nodes == 0 || samples == 0 {
        eprintln!("--nodes 和 --samples 必须大于 0");
        process::exit(2);
    }

    let started = Instant::now();
    let source = build_target_sized_source_expr(target_nodes);
    let source_nodes = source_expr_node_count(&source);
    let options = PipelineOptions {
        eval_policy: EvalPolicy::relaxed(),
        ..PipelineOptions::default()
    };
    let profiled = PipelineBuilder::new()
        .with_options(options)
        .compile_source_profiled(source)
        .unwrap_or_else(|err| {
            eprintln!("profiled compile failed: {err}");
            process::exit(1);
        });
    let total_ms = started.elapsed().as_secs_f64() * 1000.0;
    let metrics = profiled.metrics;

    println!(
        "{{\"schema_version\":1,\
         \"target_nodes\":{},\
         \"samples\":{},\
         \"source_nodes\":{},\
         \"optimized_source_nodes\":{},\
         \"expr_nodes\":{},\
         \"expr_depth\":{},\
         \"expr_unique_subexpressions\":{},\
         \"bytecode_instructions\":{},\
         \"compile_total_ms\":{:.6},\
         \"probe_total_ms\":{:.6}}}",
        target_nodes,
        samples,
        metrics.input_source_nodes.max(source_nodes),
        metrics.optimized_source_nodes,
        metrics.expr_nodes,
        metrics.expr_depth,
        metrics.expr_unique_subexpressions,
        metrics.bytecode_instructions.unwrap_or(0),
        metrics.total.as_secs_f64() * 1000.0,
        total_ms
    );
}
