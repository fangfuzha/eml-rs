use eml_rs::core::EvalPolicy;
use eml_rs::verify::{
    verify_against_complex_ref_parallel_with_policy, verify_against_complex_ref_with_policy,
    verify_against_real_ref_parallel_with_policy, verify_against_real_ref_with_policy,
    VerifyParallelism,
};
use num_complex::Complex64;

fn positive_complex_samples(n: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            vec![
                Complex64::new(0.1 + (i as f64) * 0.01, 0.0),
                Complex64::new(0.3 + (i as f64) * 0.02, 0.0),
            ]
        })
        .collect()
}

fn positive_real_samples(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| vec![0.05 + (i as f64) * 0.005, 0.2 + (i as f64) * 0.01])
        .collect()
}

#[test]
fn parallel_complex_verification_matches_serial_report() {
    let pipeline = eml_rs::api::PipelineBuilder::new()
        .compile_str("exp(x0) - log(x1 + 2)")
        .unwrap();
    let samples = positive_complex_samples(256);
    let policy = EvalPolicy::relaxed();
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial =
        verify_against_complex_ref_with_policy(pipeline.expr(), &samples, 1e-12, &policy, |vars| {
            vars[0].exp() - (vars[1] + Complex64::new(2.0, 0.0)).ln()
        });
    let parallel = verify_against_complex_ref_parallel_with_policy(
        pipeline.expr(),
        &samples,
        1e-12,
        &policy,
        parallelism,
        |vars| vars[0].exp() - (vars[1] + Complex64::new(2.0, 0.0)).ln(),
    );

    assert_eq!(parallel, serial);
    assert!(parallel.all_passed(), "{parallel:?}");
}

#[test]
fn parallel_real_verification_matches_serial_report() {
    let pipeline = eml_rs::api::PipelineBuilder::new()
        .compile_str("exp(x0) - log(x1 + 2)")
        .unwrap();
    let samples = positive_real_samples(256);
    let policy = EvalPolicy::relaxed();
    let parallelism = VerifyParallelism {
        workers: 4,
        min_samples_per_worker: 1,
    };

    let serial = verify_against_real_ref_with_policy(
        pipeline.expr(),
        &samples,
        1e-12,
        1e-12,
        &policy,
        |vars| vars[0].exp() - (vars[1] + 2.0).ln(),
    );
    let parallel = verify_against_real_ref_parallel_with_policy(
        pipeline.expr(),
        &samples,
        1e-12,
        1e-12,
        &policy,
        parallelism,
        |vars| vars[0].exp() - (vars[1] + 2.0).ln(),
    );

    assert_eq!(parallel, serial);
    assert!(parallel.all_passed(), "{parallel:?}");
}
