use eml_rs::bytecode::BytecodeProgram;
use eml_rs::core::EvalPolicy;
use eml_rs::ir::{eval_rpn_complex_with_policy, Expr};
use num_complex::Complex64;

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
