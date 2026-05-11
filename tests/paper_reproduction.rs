use eml_rs::core::EvalPolicy;
use eml_rs::lowering::{
    eval_lowered_expr_complex, eval_source_expr_complex, lower_to_eml, lower_to_lowered_eml,
    parse_source_expr, LoweredExpr, SourceExpr,
};
use num_complex::Complex64;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
struct PaperSample {
    region: &'static str,
    vars: [Complex64; 2],
}

#[derive(Debug, Clone, Copy)]
struct PaperWitnessCase {
    name: &'static str,
    catalog_name: &'static str,
    catalog_section: &'static str,
    source_formula: &'static str,
    witness_formula: &'static str,
    tolerance: f64,
    build_witness: fn() -> LoweredExpr,
}

/// Builds the EML constant `1`.
fn one() -> LoweredExpr {
    LoweredExpr::one()
}

/// Builds the first input variable.
fn var_x() -> LoweredExpr {
    LoweredExpr::var(0)
}

/// Builds the second input variable.
fn var_y() -> LoweredExpr {
    LoweredExpr::var(1)
}

/// Builds the paper witness `exp(z) = eml(z, 1)`.
fn eml_exp_witness(argument: LoweredExpr) -> LoweredExpr {
    LoweredExpr::eml(argument, one())
}

/// Builds the paper witness `ln(z) = eml(1, eml(eml(1, z), 1))`.
fn eml_log_witness(argument: LoweredExpr) -> LoweredExpr {
    LoweredExpr::eml(
        one(),
        LoweredExpr::eml(LoweredExpr::eml(one(), argument), one()),
    )
}

/// Builds the direct subtraction witness from the EML definition.
fn eml_sub_witness(minuend: LoweredExpr, subtrahend: LoweredExpr) -> LoweredExpr {
    LoweredExpr::eml(eml_log_witness(minuend), eml_exp_witness(subtrahend))
}

/// Builds the current lowering witness for unary negation.
fn eml_neg_witness(argument: LoweredExpr) -> LoweredExpr {
    eml_sub_witness(eml_sub_witness(one(), argument), one())
}

/// Builds the current lowering witness for addition.
fn eml_add_witness(lhs: LoweredExpr, rhs: LoweredExpr) -> LoweredExpr {
    eml_sub_witness(lhs, eml_neg_witness(rhs))
}

/// Builds the current lowering witness for reciprocal.
fn eml_inv_witness(argument: LoweredExpr) -> LoweredExpr {
    eml_exp_witness(eml_neg_witness(eml_log_witness(argument)))
}

/// Builds the current lowering witness for multiplication.
fn eml_mul_witness(lhs: LoweredExpr, rhs: LoweredExpr) -> LoweredExpr {
    eml_exp_witness(eml_add_witness(eml_log_witness(lhs), eml_log_witness(rhs)))
}

/// Builds the current lowering witness for division.
fn eml_div_witness(lhs: LoweredExpr, rhs: LoweredExpr) -> LoweredExpr {
    eml_mul_witness(lhs, eml_inv_witness(rhs))
}

/// Builds the current lowering witness for power.
fn eml_pow_witness(base: LoweredExpr, exponent: LoweredExpr) -> LoweredExpr {
    eml_exp_witness(eml_mul_witness(exponent, eml_log_witness(base)))
}

/// Builds the current lowering witness for square root.
fn eml_sqrt_witness(argument: LoweredExpr) -> LoweredExpr {
    let two = eml_add_witness(one(), one());
    eml_pow_witness(argument, eml_inv_witness(two))
}

/// Builds the EML constant `2`.
fn two() -> LoweredExpr {
    eml_add_witness(one(), one())
}

/// Builds the representative witness for `exp(x)`.
fn witness_exp() -> LoweredExpr {
    eml_exp_witness(var_x())
}

/// Builds the representative witness for `ln(x)`.
fn witness_log() -> LoweredExpr {
    eml_log_witness(var_x())
}

/// Builds the representative witness for `x + y`.
fn witness_add() -> LoweredExpr {
    eml_add_witness(var_x(), var_y())
}

/// Builds the representative witness for `x - y`.
fn witness_subtract() -> LoweredExpr {
    eml_sub_witness(var_x(), var_y())
}

/// Builds the representative witness for `x * y`.
fn witness_multiply() -> LoweredExpr {
    eml_mul_witness(var_x(), var_y())
}

/// Builds the representative witness for `x / y`.
fn witness_divide() -> LoweredExpr {
    eml_div_witness(var_x(), var_y())
}

/// Builds the representative witness for `pow(x, y)`.
fn witness_pow() -> LoweredExpr {
    eml_pow_witness(var_x(), var_y())
}

/// Builds the representative witness for `half(x)`.
fn witness_half() -> LoweredExpr {
    eml_div_witness(var_x(), two())
}

/// Builds the representative witness for `inv(x)`.
fn witness_inv() -> LoweredExpr {
    eml_inv_witness(var_x())
}

/// Builds the representative witness for `sqr(x)`.
fn witness_sqr() -> LoweredExpr {
    eml_mul_witness(var_x(), var_x())
}

/// Builds the representative witness for `asinh(x)`.
fn witness_asinh() -> LoweredExpr {
    let squared = eml_mul_witness(var_x(), var_x());
    eml_log_witness(eml_add_witness(
        var_x(),
        eml_sqrt_witness(eml_add_witness(squared, one())),
    ))
}

/// Builds the representative witness for `acosh(x)`.
fn witness_acosh() -> LoweredExpr {
    let left = eml_sqrt_witness(eml_sub_witness(var_x(), one()));
    let right = eml_sqrt_witness(eml_add_witness(var_x(), one()));
    eml_log_witness(eml_add_witness(var_x(), eml_mul_witness(left, right)))
}

/// Builds the representative witness for `atanh(x)`.
fn witness_atanh() -> LoweredExpr {
    let numerator = eml_sub_witness(
        eml_log_witness(eml_add_witness(one(), var_x())),
        eml_log_witness(eml_sub_witness(one(), var_x())),
    );
    eml_div_witness(numerator, eml_add_witness(one(), one()))
}

/// Builds the representative witness for `hypot(x, y)`.
fn witness_hypot() -> LoweredExpr {
    let x2 = eml_mul_witness(var_x(), var_x());
    let y2 = eml_mul_witness(var_y(), var_y());
    eml_sqrt_witness(eml_add_witness(x2, y2))
}

/// Builds the representative witness for `avg(x, y)`.
fn witness_avg() -> LoweredExpr {
    eml_div_witness(eml_add_witness(var_x(), var_y()), two())
}

/// Builds the representative witness for arbitrary-base `log_x(y)`.
fn witness_log_base() -> LoweredExpr {
    eml_div_witness(eml_log_witness(var_y()), eml_log_witness(var_x()))
}

/// Returns deterministic samples covering the paper-fidelity domains.
fn paper_samples() -> [PaperSample; 4] {
    [
        PaperSample {
            region: "positive-real-axis",
            vars: [Complex64::new(1.7, 0.0), Complex64::new(0.8, 0.0)],
        },
        PaperSample {
            region: "negative-real-axis",
            vars: [Complex64::new(-1.7, 0.0), Complex64::new(-0.8, 0.0)],
        },
        PaperSample {
            region: "zero-neighborhood",
            vars: [Complex64::new(0.05, 0.02), Complex64::new(-0.25, 0.03)],
        },
        PaperSample {
            region: "complex-plane",
            vars: [Complex64::new(0.8, 0.6), Complex64::new(-0.4, 1.1)],
        },
    ]
}

/// Returns the first completeness harness cases for representative witnesses.
fn witness_cases() -> [PaperWitnessCase; 16] {
    [
        PaperWitnessCase {
            name: "exp",
            catalog_name: "exp",
            catalog_section: "unaryFunctions",
            source_formula: "exp(x)",
            witness_formula: "eml(x, 1)",
            tolerance: 1e-9,
            build_witness: witness_exp,
        },
        PaperWitnessCase {
            name: "ln",
            catalog_name: "log",
            catalog_section: "unaryFunctions",
            source_formula: "log(x)",
            witness_formula: "eml(1, eml(eml(1, x), 1))",
            tolerance: 2e-8,
            build_witness: witness_log,
        },
        PaperWitnessCase {
            name: "add",
            catalog_name: "add",
            catalog_section: "binaryOperations",
            source_formula: "x + y",
            witness_formula: "x - (-y)",
            tolerance: 2e-8,
            build_witness: witness_add,
        },
        PaperWitnessCase {
            name: "subtract",
            catalog_name: "subtract",
            catalog_section: "binaryOperations",
            source_formula: "x - y",
            witness_formula: "eml(ln(x), exp(y))",
            tolerance: 2e-8,
            build_witness: witness_subtract,
        },
        PaperWitnessCase {
            name: "multiply",
            catalog_name: "multiply",
            catalog_section: "binaryOperations",
            source_formula: "x * y",
            witness_formula: "exp(ln(x) + ln(y))",
            tolerance: 5e-8,
            build_witness: witness_multiply,
        },
        PaperWitnessCase {
            name: "divide",
            catalog_name: "divide",
            catalog_section: "binaryOperations",
            source_formula: "x / y",
            witness_formula: "x * (1 / y)",
            tolerance: 5e-8,
            build_witness: witness_divide,
        },
        PaperWitnessCase {
            name: "pow",
            catalog_name: "pow",
            catalog_section: "binaryOperations",
            source_formula: "pow(x, y)",
            witness_formula: "exp(y * ln(x))",
            tolerance: 5e-7,
            build_witness: witness_pow,
        },
        PaperWitnessCase {
            name: "half",
            catalog_name: "half",
            catalog_section: "unaryFunctions",
            source_formula: "half(x)",
            witness_formula: "x / 2",
            tolerance: 2e-8,
            build_witness: witness_half,
        },
        PaperWitnessCase {
            name: "inv",
            catalog_name: "inv",
            catalog_section: "unaryFunctions",
            source_formula: "inv(x)",
            witness_formula: "exp(-ln(x))",
            tolerance: 5e-8,
            build_witness: witness_inv,
        },
        PaperWitnessCase {
            name: "sqr",
            catalog_name: "sqr",
            catalog_section: "unaryFunctions",
            source_formula: "sqr(x)",
            witness_formula: "x * x",
            tolerance: 5e-7,
            build_witness: witness_sqr,
        },
        PaperWitnessCase {
            name: "asinh",
            catalog_name: "asinh",
            catalog_section: "unaryFunctions",
            source_formula: "asinh(x)",
            witness_formula: "log(x + sqrt(x^2 + 1))",
            tolerance: 2e-6,
            build_witness: witness_asinh,
        },
        PaperWitnessCase {
            name: "acosh",
            catalog_name: "acosh",
            catalog_section: "unaryFunctions",
            source_formula: "acosh(x)",
            witness_formula: "log(x + sqrt(x - 1) * sqrt(x + 1))",
            tolerance: 2e-6,
            build_witness: witness_acosh,
        },
        PaperWitnessCase {
            name: "atanh",
            catalog_name: "atanh",
            catalog_section: "unaryFunctions",
            source_formula: "atanh(x)",
            witness_formula: "(log(1 + x) - log(1 - x)) / 2",
            tolerance: 2e-6,
            build_witness: witness_atanh,
        },
        PaperWitnessCase {
            name: "hypot",
            catalog_name: "hypot",
            catalog_section: "binaryOperations",
            source_formula: "hypot(x, y)",
            witness_formula: "sqrt(x^2 + y^2)",
            tolerance: 5e-6,
            build_witness: witness_hypot,
        },
        PaperWitnessCase {
            name: "log_base",
            catalog_name: "log_base",
            catalog_section: "binaryOperations",
            source_formula: "log_x(x, y)",
            witness_formula: "log(y) / log(x)",
            tolerance: 5e-6,
            build_witness: witness_log_base,
        },
        PaperWitnessCase {
            name: "avg",
            catalog_name: "avg",
            catalog_section: "binaryOperations",
            source_formula: "avg(x, y)",
            witness_formula: "(x + y) / 2",
            tolerance: 2e-8,
            build_witness: witness_avg,
        },
    ]
}

/// Parses a source formula and includes the case name in panic output.
fn parse_case_source(case: &PaperWitnessCase) -> SourceExpr {
    parse_source_expr(case.source_formula)
        .unwrap_or_else(|err| panic!("{} source parse failed: {err}", case.name))
}

/// Compares complex outputs with tolerance and matching non-finite handling.
fn complex_values_match(actual: Complex64, expected: Complex64, tolerance: f64) -> bool {
    if actual.re.is_finite()
        && actual.im.is_finite()
        && expected.re.is_finite()
        && expected.im.is_finite()
    {
        return (actual - expected).norm() <= tolerance;
    }

    scalar_values_match(actual.re, expected.re, tolerance)
        && scalar_values_match(actual.im, expected.im, tolerance)
}

/// Compares scalar outputs while accepting matching NaN and infinity values.
fn scalar_values_match(actual: f64, expected: f64, tolerance: f64) -> bool {
    if actual.is_nan() || expected.is_nan() {
        return actual.is_nan() && expected.is_nan();
    }
    if actual.is_infinite() || expected.is_infinite() {
        return actual == expected;
    }
    (actual - expected).abs() <= tolerance
}

/// Asserts two complex outputs are within the case tolerance.
fn assert_close(
    case: &PaperWitnessCase,
    sample: &PaperSample,
    actual_label: &str,
    actual: Complex64,
    expected_label: &str,
    expected: Complex64,
) {
    assert!(
        complex_values_match(actual, expected, case.tolerance),
        "case={} region={} witness={} {actual_label}={actual:?} {expected_label}={expected:?} tolerance={}",
        case.name,
        sample.region,
        case.witness_formula,
        case.tolerance
    );
}

/// Looks up the status for a named catalog entry.
fn catalog_status(catalog: &Value, section: &str, name: &str) -> Option<String> {
    catalog
        .get(section)?
        .as_array()?
        .iter()
        .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|entry| entry.get("status"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// Replays representative `VerifyBaseSet` witness chains against lowering and source references.
#[test]
fn paper_basis_verify_base_set_witness_chains_replay() {
    let relaxed_policy = EvalPolicy::relaxed();

    for case in witness_cases() {
        let source = parse_case_source(&case);
        let lowered_witness = (case.build_witness)();
        let lowered_from_source = lower_to_lowered_eml(&source)
            .unwrap_or_else(|err| panic!("{} lowering failed: {err}", case.name));
        let runtime_from_source = lower_to_eml(&source)
            .unwrap_or_else(|err| panic!("{} runtime lowering failed: {err}", case.name));

        for sample in paper_samples() {
            let source_value = eval_source_expr_complex(&source, &sample.vars)
                .unwrap_or_else(|err| panic!("{} source eval failed: {err}", case.name));
            let pure_witness_value = eval_lowered_expr_complex(&lowered_witness, &sample.vars)
                .unwrap_or_else(|err| panic!("{} pure witness eval failed: {err}", case.name));
            let lowered_value = eval_lowered_expr_complex(&lowered_from_source, &sample.vars)
                .unwrap_or_else(|err| panic!("{} lowered eval failed: {err}", case.name));
            let runtime_value = runtime_from_source
                .eval_complex_with_policy(&sample.vars, &relaxed_policy)
                .unwrap_or_else(|err| panic!("{} runtime eval failed: {err}", case.name));

            assert_close(
                &case,
                &sample,
                "pure_witness",
                pure_witness_value,
                "source_reference",
                source_value,
            );
            assert_close(
                &case,
                &sample,
                "lowering_result",
                lowered_value,
                "source_reference",
                source_value,
            );
            assert_close(
                &case,
                &sample,
                "runtime_result",
                runtime_value,
                "source_reference",
                source_value,
            );
        }
    }
}

/// Ensures every representative witness is exercised over the required sample regions.
#[test]
fn paper_basis_representative_witnesses_cover_domain_regions() {
    let samples = paper_samples();
    let sample_regions = samples.map(|sample| sample.region);
    assert_eq!(
        sample_regions,
        [
            "positive-real-axis",
            "negative-real-axis",
            "zero-neighborhood",
            "complex-plane",
        ]
    );

    for case in witness_cases() {
        let source = parse_case_source(&case);
        for sample in samples {
            let value = eval_source_expr_complex(&source, &sample.vars).unwrap_or_else(|err| {
                panic!(
                    "{} source formula {} failed on {}: {err}",
                    case.name, case.source_formula, sample.region
                )
            });
            assert!(
                value.re.is_finite() && value.im.is_finite(),
                "case={} region={} produced non-finite source value {value:?}",
                case.name,
                sample.region
            );
        }
    }
}

/// Keeps the machine-readable catalog aligned with the replayed harness cases.
#[test]
fn paper_basis_catalog_records_replayed_witnesses() {
    let catalog: Value = serde_json::from_str(include_str!("../docs/paper-basis-catalog.json"))
        .expect("paper basis catalog must be valid JSON");

    for case in witness_cases() {
        let status = catalog_status(&catalog, case.catalog_section, case.catalog_name)
            .unwrap_or_else(|| panic!("missing catalog entry for {}", case.name));
        assert_eq!(
            status, "covered",
            "case={} source_formula={} catalog status should be covered",
            case.name, case.source_formula
        );
    }
}
