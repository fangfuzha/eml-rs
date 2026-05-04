use std::env;
use std::fs;
use std::process;

use eml_rs::api::{BuiltinBackend, PipelineBuilder, PipelineOptions};
use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{
    eval_source_expr_complex, lower_to_eml, parse_source_expr, source_expr_node_count, SourceExpr,
};
use eml_rs::opt::optimize_for_lowering;
use num_complex::Complex64;

fn usage() -> &'static str {
    "用法:
  eml parse <expr>
  eml lower <expr>
  eml verify <expr> --samples <samples.json> [--tolerance <f64>] [--relaxed]
  eml profile <expr> [--samples <samples.json>] [--sample-count <usize>] [--relaxed]

样本 JSON 格式: [[0.2, 1.4], [0.5, 2.0]]
表达式如果包含空格，请用引号包住。"
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn required_expr(args: &[String]) -> Result<&str, String> {
    args.get(2)
        .map(String::as_str)
        .ok_or_else(|| "missing expression argument".to_string())
}

fn pipeline_builder(args: &[String]) -> Result<PipelineBuilder, String> {
    let strict = args.iter().any(|arg| arg == "--strict");
    let relaxed = args.iter().any(|arg| arg == "--relaxed");
    if strict && relaxed {
        return Err("--strict and --relaxed cannot be used together".to_string());
    }

    let mut options = PipelineOptions::default();
    if relaxed {
        options.eval_policy = EvalPolicy::relaxed();
    }
    Ok(PipelineBuilder::new().with_options(options))
}

fn read_real_samples(path: &str) -> Result<Vec<Vec<f64>>, String> {
    let data = fs::read_to_string(path).map_err(|err| format!("failed to read samples: {err}"))?;
    let samples: Vec<Vec<f64>> =
        serde_json::from_str(&data).map_err(|err| format!("invalid samples JSON: {err}"))?;
    if samples.is_empty() || samples.iter().any(Vec::is_empty) {
        return Err("samples must be a non-empty array of non-empty arrays".to_string());
    }
    Ok(samples)
}

fn source_var_arity(expr: &SourceExpr) -> usize {
    match expr {
        SourceExpr::Var(index) => index + 1,
        SourceExpr::Int(_)
        | SourceExpr::Rational(_, _)
        | SourceExpr::ConstE
        | SourceExpr::ConstI
        | SourceExpr::ConstPi => 0,
        SourceExpr::Neg(inner)
        | SourceExpr::Exp(inner)
        | SourceExpr::Log(inner)
        | SourceExpr::Sin(inner)
        | SourceExpr::Cos(inner)
        | SourceExpr::Tan(inner)
        | SourceExpr::Sinh(inner)
        | SourceExpr::Cosh(inner)
        | SourceExpr::Tanh(inner)
        | SourceExpr::Asin(inner)
        | SourceExpr::Acos(inner)
        | SourceExpr::Atan(inner)
        | SourceExpr::Sqrt(inner)
        | SourceExpr::Sigmoid(inner)
        | SourceExpr::Softplus(inner)
        | SourceExpr::Swish(inner)
        | SourceExpr::GeluTanh(inner)
        | SourceExpr::ReluSoft(inner)
        | SourceExpr::Softsign(inner)
        | SourceExpr::Mish(inner) => source_var_arity(inner),
        SourceExpr::Add(lhs, rhs)
        | SourceExpr::Sub(lhs, rhs)
        | SourceExpr::Mul(lhs, rhs)
        | SourceExpr::Div(lhs, rhs)
        | SourceExpr::Pow(lhs, rhs)
        | SourceExpr::Elu(lhs, rhs)
        | SourceExpr::LeakyRelu(lhs, rhs) => source_var_arity(lhs).max(source_var_arity(rhs)),
    }
}

fn default_profile_samples(source: &SourceExpr, sample_count: usize) -> Vec<Vec<f64>> {
    let arity = source_var_arity(source);
    let sample_count = sample_count.max(1);
    (0..sample_count)
        .map(|sample_index| {
            (0..arity)
                .map(|var_index| 0.5 + (var_index as f64 + 1.0) * 0.1 + sample_index as f64 * 0.001)
                .collect()
        })
        .collect()
}

fn real_to_complex_samples(samples: &[Vec<f64>]) -> Vec<Vec<Complex64>> {
    samples
        .iter()
        .map(|sample| {
            sample
                .iter()
                .copied()
                .map(|value| Complex64::new(value, 0.0))
                .collect()
        })
        .collect()
}

fn backend_name(backend: BuiltinBackend) -> &'static str {
    match backend {
        BuiltinBackend::Tree => "tree",
        BuiltinBackend::Rpn => "rpn",
        BuiltinBackend::Bytecode => "bytecode",
    }
}

fn finite_complex(value: Complex64) -> bool {
    value.re.is_finite() && value.im.is_finite()
}

fn cmd_parse(args: &[String]) -> Result<(), String> {
    let expr = parse_source_expr(required_expr(args)?).map_err(|err| err.to_string())?;
    println!("SourceExpr:\n{expr:#?}");
    println!("source_nodes={}", source_expr_node_count(&expr));
    Ok(())
}

fn cmd_lower(args: &[String]) -> Result<(), String> {
    let source = parse_source_expr(required_expr(args)?).map_err(|err| err.to_string())?;
    let optimized = optimize_for_lowering(&source);
    let expr = lower_to_eml(&optimized).map_err(|err| err.to_string())?;
    println!("Expr:\n{expr:#?}");
    println!("ExprStats: {:?}", expr.stats());
    println!("source_nodes={}", source_expr_node_count(&source));
    println!(
        "optimized_source_nodes={}",
        source_expr_node_count(&optimized)
    );
    Ok(())
}

fn cmd_verify(args: &[String]) -> Result<(), String> {
    let expr_text = required_expr(args)?;
    let samples_path =
        arg_value(args, "--samples").ok_or_else(|| "missing --samples <path>".to_string())?;
    let tolerance = arg_value(args, "--tolerance")
        .as_deref()
        .unwrap_or("1e-8")
        .parse::<f64>()
        .map_err(|err| format!("invalid --tolerance: {err}"))?;
    let samples = real_to_complex_samples(&read_real_samples(&samples_path)?);
    let pipeline = pipeline_builder(args)?
        .compile_str(expr_text)
        .map_err(|err| err.to_string())?;
    let mut all_passed = true;

    for backend in [
        BuiltinBackend::Tree,
        BuiltinBackend::Rpn,
        BuiltinBackend::Bytecode,
    ] {
        let mut total = 0usize;
        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut max_abs_error = 0.0f64;
        for vars in &samples {
            total += 1;
            let expected = eval_source_expr_complex(pipeline.optimized_source(), vars)
                .map_err(|err| err.to_string())?;
            if !finite_complex(expected) {
                failed += 1;
                continue;
            }
            let actual = pipeline
                .eval_complex(backend, vars)
                .map_err(|err| err.to_string())?;
            let err = (actual - expected).norm();
            max_abs_error = max_abs_error.max(err);
            if err <= tolerance {
                passed += 1;
            } else {
                failed += 1;
            }
        }

        let backend_passed = total > 0 && failed == 0;
        all_passed &= backend_passed;
        println!(
            "backend={} passed={} total={} passed_samples={} failed={} max_abs_error={:.6e}",
            backend_name(backend),
            backend_passed,
            total,
            passed,
            failed,
            max_abs_error
        );
    }

    println!("passed={all_passed}");

    if all_passed {
        Ok(())
    } else {
        Err("verification failed".to_string())
    }
}

fn cmd_profile(args: &[String]) -> Result<(), String> {
    let profiled = pipeline_builder(args)?
        .compile_str_profiled(required_expr(args)?)
        .map_err(|err| err.to_string())?;
    let metrics = profiled.metrics;
    println!(
        "compile_total_ms={:.6}",
        metrics.total.as_secs_f64() * 1000.0
    );
    println!("parse_ms={:.6}", metrics.parse.as_secs_f64() * 1000.0);
    println!("simplify_ms={:.6}", metrics.simplify.as_secs_f64() * 1000.0);
    println!("lowering_ms={:.6}", metrics.lowering.as_secs_f64() * 1000.0);
    println!(
        "rpn_build_ms={:.6}",
        metrics.rpn_build.as_secs_f64() * 1000.0
    );
    println!(
        "bytecode_build_ms={:.6}",
        metrics
            .bytecode_build
            .map(|duration| duration.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    );
    println!("source_nodes={}", metrics.input_source_nodes);
    println!("optimized_source_nodes={}", metrics.optimized_source_nodes);
    println!("expr_nodes={}", metrics.expr_nodes);
    println!("expr_depth={}", metrics.expr_depth);
    println!(
        "bytecode_instructions={}",
        metrics.bytecode_instructions.unwrap_or(0)
    );
    let sample_count = arg_value(args, "--sample-count")
        .as_deref()
        .unwrap_or("16")
        .parse::<usize>()
        .map_err(|err| format!("invalid --sample-count: {err}"))?;
    let real_samples = if let Some(path) = arg_value(args, "--samples") {
        read_real_samples(&path)?
    } else {
        default_profile_samples(profiled.pipeline.optimized_source(), sample_count)
    };
    let samples = real_to_complex_samples(&real_samples);
    for backend in [
        BuiltinBackend::Tree,
        BuiltinBackend::Rpn,
        BuiltinBackend::Bytecode,
    ] {
        let eval_metrics = profiled
            .pipeline
            .profile_eval_complex_batch(backend, &samples);
        match eval_metrics {
            Ok(eval_metrics) => {
                println!(
                    "eval_backend={} eval_total_ms={:.6} eval_per_sample_us={:.6} eval_samples={}",
                    backend_name(eval_metrics.backend),
                    eval_metrics.total.as_secs_f64() * 1000.0,
                    eval_metrics.per_sample.as_secs_f64() * 1_000_000.0,
                    eval_metrics.samples
                );
            }
            Err(err) => {
                println!("eval_backend={} eval_error={err}", backend_name(backend));
            }
        }
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    match args.get(1).map(String::as_str) {
        Some("parse") => cmd_parse(args),
        Some("lower") => cmd_lower(args),
        Some("verify") => cmd_verify(args),
        Some("profile") => cmd_profile(args),
        Some("-h" | "--help") | None => {
            println!("{}", usage());
            Ok(())
        }
        Some(other) => Err(format!("unknown command: {other}\n{}", usage())),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(err) = run(&args) {
        eprintln!("{err}");
        process::exit(1);
    }
}
