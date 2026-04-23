#![no_main]

use eml_rs::bytecode::BytecodeProgram;
use eml_rs::core::EvalPolicy;
use eml_rs::ir::{eval_rpn_complex_with_policy, Expr};
use libfuzzer_sys::fuzz_target;
use num_complex::Complex64;

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

fuzz_target!(|data: &[u8]| {
    let mut index = 0usize;
    let expr = decode_expr(data, &mut index, 0);
    let rpn = expr.to_rpn_vec();
    let prog = match BytecodeProgram::from_expr_with_policy(&expr, &EvalPolicy::relaxed()) {
        Ok(v) => v,
        Err(_) => return,
    };
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(0.5, 0.1),
        Complex64::new(1.2, -0.3),
        Complex64::new(2.0, 0.0),
    ];
    let tree = expr.eval_complex_with_policy(&vars, &EvalPolicy::relaxed()).ok();
    let rpn_v = eval_rpn_complex_with_policy(&rpn, &vars, &EvalPolicy::relaxed()).ok();
    let bytecode = prog.eval_complex_with_policy(&vars, &EvalPolicy::relaxed()).ok();

    if let (Some(tree), Some(rpn_v), Some(bytecode)) = (tree, rpn_v, bytecode) {
        assert!((tree - rpn_v).norm() <= 1e-8);
        assert!((tree - bytecode).norm() <= 1e-8);
    }
});
