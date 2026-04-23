#![no_main]

use eml_rs::lowering::{
    eval_source_expr_complex, parse_source_expr, simplify_source_expr, symbolic_derivative,
};
use libfuzzer_sys::fuzz_target;
use num_complex::Complex64;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(source) = parse_source_expr(input) else {
        return;
    };
    let deriv = symbolic_derivative(&source, 0);
    let simplified = simplify_source_expr(&deriv);
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(0.5, 0.1),
        Complex64::new(1.2, -0.3),
        Complex64::new(2.0, 0.0),
    ];
    let _ = eval_source_expr_complex(&deriv, &vars);
    let _ = eval_source_expr_complex(&simplified, &vars);
});
