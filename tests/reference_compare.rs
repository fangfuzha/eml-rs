use eml_rs::bytecode::BytecodeProgram;
use eml_rs::core::{EvalPolicy, LogBranchPolicy};
use eml_rs::ir::{eval_rpn_complex, Expr};
use eml_rs::lowering::{eval_source_expr_complex, lower_to_eml, parse_source_expr};
use eml_rs::opt::{estimate_cost, optimize_for_lowering};
use eml_rs::verify::{verify_against_complex_ref, verify_against_real_ref};
use num_complex::Complex64;

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
