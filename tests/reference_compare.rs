use eml_rs::ir::{eval_rpn_complex, Expr};
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
