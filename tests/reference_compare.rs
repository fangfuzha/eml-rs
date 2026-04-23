use eml_rs::bytecode::BytecodeProgram;
use eml_rs::core::{EvalPolicy, LogBranchPolicy};
use eml_rs::ir::{eval_rpn_complex, Expr};
use eml_rs::lowering::{
    batch_cross_entropy_mean_template, cross_entropy_template, delower_to_source,
    eval_lowered_expr_complex, eval_source_expr_complex, lower_to_eml, lower_to_lowered_eml,
    parse_source_expr, softmax_template, symbolic_derivative,
};
use eml_rs::opt::{estimate_cost, optimize_for_lowering};
use eml_rs::verify::{verify_against_complex_ref, verify_against_real_ref};
use num_complex::Complex64;

fn finite_diff_real(
    expr: &eml_rs::lowering::SourceExpr,
    vars: &[Complex64],
    var_index: usize,
    h: f64,
) -> f64 {
    let mut plus = vars.to_vec();
    let mut minus = vars.to_vec();
    plus[var_index].re += h;
    minus[var_index].re -= h;
    let f_plus = eval_source_expr_complex(expr, &plus).unwrap();
    let f_minus = eval_source_expr_complex(expr, &minus).unwrap();
    ((f_plus - f_minus) / Complex64::new(2.0 * h, 0.0)).re
}

#[test]
fn eml_exp_matches_complex_exp() {
    let expr = Expr::exp(Expr::var(0));
    let samples = vec![
        vec![Complex64::new(-2.0, 0.0)],
        vec![Complex64::new(-0.5, 0.1)],
        vec![Complex64::new(0.0, -0.6)],
        vec![Complex64::new(1.2, 0.3)],
    ];

    let report = verify_against_complex_ref(&expr, &samples, 1e-12, |vars| vars[0].exp());
    assert!(report.all_passed(), "{report:?}");
}

#[test]
fn eml_log_formula_matches_real_ln() {
    let expr = Expr::ln(Expr::var(0));
    let samples = vec![
        vec![0.1],
        vec![0.5],
        vec![1.0],
        vec![2.0],
        vec![10.0],
        vec![100.0],
    ];

    let report = verify_against_real_ref(&expr, &samples, 1e-12, 1e-11, |vars| vars[0].ln());
    assert!(report.all_passed(), "{report:?}");
}

#[test]
fn rpn_and_tree_evaluation_agree() {
    let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::var(1));
    let rpn = expr.to_rpn_vec();
    let samples = vec![
        vec![Complex64::new(0.1, 0.0), Complex64::new(1.3, 0.0)],
        vec![Complex64::new(-0.4, 0.2), Complex64::new(0.8, -0.1)],
        vec![Complex64::new(0.9, -0.3), Complex64::new(2.1, 0.4)],
    ];

    for vars in samples {
        let tree = expr.eval_complex(&vars).unwrap();
        let stack = eval_rpn_complex(&rpn, &vars).unwrap();
        assert!(
            (tree - stack).norm() <= 1e-12,
            "tree={tree:?}, rpn={stack:?}, vars={vars:?}"
        );
    }
}

#[test]
fn bytecode_and_tree_evaluation_agree() {
    let source = parse_source_expr("sin(x0) + cos(x0)^2 - log(x1)").unwrap();
    let source = optimize_for_lowering(&source);
    let expr = lower_to_eml(&source).unwrap();
    let prog = BytecodeProgram::from_expr(&expr).unwrap();

    let vars = vec![Complex64::new(0.4, 0.2), Complex64::new(1.8, -0.1)];
    let relaxed = EvalPolicy::relaxed();
    let tree = expr.eval_complex_with_policy(&vars, &relaxed).unwrap();
    let bytecode = prog.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!((tree - bytecode).norm() <= 1e-8);
}

#[test]
fn source_lowering_matches_native_reference() {
    let source = parse_source_expr("exp(x0) + sin(x1) / cos(x1)").unwrap();
    let optimized = optimize_for_lowering(&source);
    assert!(estimate_cost(&optimized).score <= estimate_cost(&source).score);

    let expr = lower_to_eml(&optimized).unwrap();
    let vars = vec![Complex64::new(0.3, -0.2), Complex64::new(0.5, 0.1)];
    let ref_v = eval_source_expr_complex(&optimized, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let eml_v = expr.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!((ref_v - eml_v).norm() <= 1e-7);
}

#[test]
fn corrected_real_branch_can_be_selected() {
    let expr = Expr::ln(Expr::var(0));
    let policy = EvalPolicy {
        log_branch: LogBranchPolicy::CorrectedReal,
        ..EvalPolicy::default()
    };
    let out = expr
        .eval_complex_with_policy(&[Complex64::new(-2.0, 0.0)], &policy)
        .unwrap();
    assert!(out.im.abs() > 3.0);
}

#[test]
fn extended_elementary_functions_match_reference() {
    let source = parse_source_expr("tan(x0) + sinh(x0) + cosh(x0) + atan(x0) + sqrt(x1)").unwrap();
    let expr = lower_to_eml(&source).unwrap();
    let vars = vec![Complex64::new(0.2, 0.1), Complex64::new(1.4, -0.2)];
    let ref_v = eval_source_expr_complex(&source, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let eml_v = expr.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!(
        (ref_v - eml_v).norm() <= 2e-6,
        "ref={ref_v:?}, eml={eml_v:?}"
    );
}

#[test]
fn ai_training_function_family_is_lowerable_and_evaluable() {
    let source =
        parse_source_expr("sigmoid(x0) + softplus(x0) + swish(x0) + gelu(x0) + relu(x0)").unwrap();
    let expr = lower_to_eml(&source).unwrap();
    let vars = vec![Complex64::new(0.35, 0.0)];
    let ref_v = eval_source_expr_complex(&source, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let eml_v = expr.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!(
        (ref_v - eml_v).norm() <= 2e-5,
        "ref={ref_v:?}, eml={eml_v:?}"
    );
}

#[test]
fn added_activation_family_is_lowerable_and_evaluable() {
    let source =
        parse_source_expr("elu(x0) + leaky_relu(x0,0.05) + softsign(x0) + mish(x0)").unwrap();
    let expr = lower_to_eml(&source).unwrap();
    let vars = vec![Complex64::new(0.35, 0.0)];
    let ref_v = eval_source_expr_complex(&source, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let eml_v = expr.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!(
        (ref_v - eml_v).norm() <= 2e-5,
        "ref={ref_v:?}, eml={eml_v:?}"
    );
}

#[test]
fn softmax_cross_entropy_vector_templates_are_lowerable() {
    let logits = vec![
        parse_source_expr("x0").unwrap(),
        parse_source_expr("x1 + 1").unwrap(),
        parse_source_expr("x2 - 1").unwrap(),
    ];
    let probs = softmax_template(&logits).unwrap();
    assert_eq!(probs.len(), 3);
    let ce = cross_entropy_template(&logits, 1).unwrap();
    let ce_eml = lower_to_eml(&ce).unwrap();
    let vars = vec![
        Complex64::new(0.3, 0.0),
        Complex64::new(1.1, 0.0),
        Complex64::new(-0.4, 0.0),
    ];
    let ce_ref = eval_source_expr_complex(&ce, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let ce_eval = ce_eml.eval_complex_with_policy(&vars, &relaxed).unwrap();
    assert!((ce_ref - ce_eval).norm() <= 5e-6);
}

#[test]
fn symbolic_derivative_matches_finite_difference() {
    let source = parse_source_expr("softplus(x0) + mish(x0)").unwrap();
    let deriv = symbolic_derivative(&source, 0);
    let vars = [Complex64::new(0.35, 0.0)];
    let analytic = eval_source_expr_complex(&deriv, &vars).unwrap().re;
    let numeric = finite_diff_real(&source, &vars, 0, 1e-6);
    assert!((analytic - numeric).abs() <= 5e-3);
}

#[test]
fn batch_cross_entropy_mean_template_is_lowerable() {
    let batch_logits = vec![
        vec![
            parse_source_expr("x0").unwrap(),
            parse_source_expr("x1 + 0.5").unwrap(),
            parse_source_expr("x2 - 0.25").unwrap(),
        ],
        vec![
            parse_source_expr("x3").unwrap(),
            parse_source_expr("x4 + 1").unwrap(),
            parse_source_expr("x5").unwrap(),
        ],
    ];
    let targets = vec![0usize, 2usize];
    let mean_ce = batch_cross_entropy_mean_template(&batch_logits, &targets).unwrap();
    let mean_ce_eml = lower_to_eml(&mean_ce).unwrap();
    let vars = vec![
        Complex64::new(0.3, 0.0),
        Complex64::new(1.1, 0.0),
        Complex64::new(-0.4, 0.0),
        Complex64::new(0.7, 0.0),
        Complex64::new(-0.2, 0.0),
        Complex64::new(0.4, 0.0),
    ];
    let ref_v = eval_source_expr_complex(&mean_ce, &vars).unwrap();
    let relaxed = EvalPolicy::relaxed();
    let eval_v = mean_ce_eml
        .eval_complex_with_policy(&vars, &relaxed)
        .unwrap();
    assert!((ref_v - eval_v).norm() <= 1e-5);
}

#[test]
fn delowering_backend_preserves_lowered_semantics() {
    let source = parse_source_expr("sigmoid(x0) + softplus(x1) - log(x2 + 3)").unwrap();
    let lowered = lower_to_lowered_eml(&source).unwrap();
    let raised = delower_to_source(&lowered);
    let vars = [
        Complex64::new(0.2, 0.0),
        Complex64::new(-0.3, 0.0),
        Complex64::new(0.8, 0.0),
    ];
    let lowered_v = eval_lowered_expr_complex(&lowered, &vars).unwrap();
    let raised_v = eval_source_expr_complex(&raised, &vars).unwrap();
    assert!((lowered_v - raised_v).norm() <= 1e-12);
}
