#![no_main]

use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{eval_source_expr_complex, lower_to_eml, parse_source_expr};
use libfuzzer_sys::fuzz_target;
use num_complex::Complex64;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(source) = parse_source_expr(input) else {
        return;
    };
    let Ok(expr) = lower_to_eml(&source) else {
        return;
    };
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(0.5, 0.1),
        Complex64::new(1.2, -0.3),
        Complex64::new(2.0, 0.0),
    ];
    let _ = eval_source_expr_complex(&source, &vars);
    let _ = expr.eval_complex_with_policy(&vars, &EvalPolicy::relaxed());
});
