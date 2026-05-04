use eml_rs::bytecode::BytecodeProgram;
use eml_rs::core::EvalPolicy;
use eml_rs::ir::{eval_rpn_complex_with_policy, Expr};
use eml_rs::lowering::{
    eval_source_expr_complex, parse_source_expr, simplify_source_expr, symbolic_derivative,
};
use num_complex::Complex64;
use std::fs;
use std::path::PathBuf;

fn same_or_close(lhs: Complex64, rhs: Complex64, tol: f64) -> bool {
    if lhs.re.is_finite() && lhs.im.is_finite() && rhs.re.is_finite() && rhs.im.is_finite() {
        return (lhs - rhs).norm() <= tol;
    }

    fn scalar_eq(lhs: f64, rhs: f64, tol: f64) -> bool {
        if lhs.is_nan() || rhs.is_nan() {
            return lhs.is_nan() && rhs.is_nan();
        }
        if lhs.is_infinite() || rhs.is_infinite() {
            return lhs == rhs;
        }
        (lhs - rhs).abs() <= tol
    }

    scalar_eq(lhs.re, rhs.re, tol) && scalar_eq(lhs.im, rhs.im, tol)
}

fn corpus_file(target: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("corpus")
        .join(target)
        .join(name)
}

fn decode_expr(data: &[u8], index: &mut usize, depth: usize) -> Expr {
    if depth >= 6 || *index >= data.len() {
        return Expr::one();
    }

    let tag = data[*index] % 3;
    *index += 1;
    match tag {
        0 => Expr::one(),
        1 => {
            if *index >= data.len() {
                Expr::var(0)
            } else {
                let var = (data[*index] % 4) as usize;
                *index += 1;
                Expr::var(var)
            }
        }
        _ => {
            let lhs = decode_expr(data, index, depth + 1);
            let rhs = decode_expr(data, index, depth + 1);
            Expr::eml(lhs, rhs)
        }
    }
}

#[test]
fn fuzz_expr_eval_consistency_regression_case() {
    let mut index = 0usize;
    let expr = decode_expr(&[182, 212, 182, 245], &mut index, 0);
    let rpn = expr.to_rpn_vec();
    let policy = EvalPolicy::relaxed();
    let prog = BytecodeProgram::from_expr_with_policy(&expr, &policy).unwrap();
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(0.5, 0.1),
        Complex64::new(1.2, -0.3),
        Complex64::new(2.0, 0.0),
    ];

    let tree = expr.eval_complex_with_policy(&vars, &policy).unwrap();
    let rpn_v = eval_rpn_complex_with_policy(&rpn, &vars, &policy).unwrap();
    let bytecode = prog.eval_complex_with_policy(&vars, &policy).unwrap();

    assert!(
        same_or_close(tree, rpn_v, 1e-8),
        "expr={expr:?}, tree={tree:?}, rpn={rpn_v:?}, bytecode={bytecode:?}"
    );
    assert!(
        same_or_close(tree, bytecode, 1e-8),
        "expr={expr:?}, tree={tree:?}, rpn={rpn_v:?}, bytecode={bytecode:?}"
    );
}

#[test]
fn parse_lower_eval_corpus_contains_large_integer_regression() {
    let path = corpus_file("parse_lower_eval", "large_integer_literal");
    let bytes = fs::read(path).unwrap();
    assert_eq!(bytes, b"515111");
}

#[test]
fn expr_eval_consistency_corpus_contains_non_finite_regression() {
    let path = corpus_file("expr_eval_consistency", "matching_non_finite_backends");
    let bytes = fs::read(path).unwrap();
    assert_eq!(bytes, [182, 212, 182, 245]);
}

#[test]
fn autodiff_simplify_corpus_contains_regression_seeds() {
    let expected = [
        ("power_compaction", "x0^8"),
        ("transcendental_product", "exp(x0) * log(x1 + 2)"),
        ("activation_mix", "softplus(x0) + mish(x0)"),
        (
            "branchy_activation_mix",
            "mish(x0) + elu(x0,0.5) + leaky_relu(x0,0.1)",
        ),
        ("pow_i64_min_neg_overflow", "x0*02^-8^21"),
    ];

    for (name, source) in expected {
        let path = corpus_file("autodiff_simplify", name);
        let bytes = fs::read(path).unwrap();
        assert_eq!(bytes, source.as_bytes(), "seed {name} drifted");
    }
}

#[test]
fn autodiff_simplify_corpus_examples_parse_and_evaluate() {
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(0.5, 0.1),
        Complex64::new(1.2, -0.3),
        Complex64::new(2.0, 0.0),
    ];

    for name in [
        "power_compaction",
        "transcendental_product",
        "activation_mix",
        "branchy_activation_mix",
        "pow_i64_min_neg_overflow",
    ] {
        let path = corpus_file("autodiff_simplify", name);
        let input = fs::read_to_string(path).unwrap();
        let source = parse_source_expr(&input).unwrap();
        let deriv = symbolic_derivative(&source, 0);
        let simplified = simplify_source_expr(&deriv);
        let _ = eval_source_expr_complex(&deriv, &vars);
        let _ = eval_source_expr_complex(&simplified, &vars);
    }
}
