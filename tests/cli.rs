use std::fs;
use std::process::Command;

fn eml_bin() -> &'static str {
    option_env!("CARGO_BIN_EXE_eml").expect("cargo should expose the eml binary path")
}

#[test]
fn cli_parse_and_lower_commands_run() {
    let parse = Command::new(eml_bin())
        .args(["parse", "exp(x0) - log(x1)"])
        .output()
        .unwrap();
    assert!(
        parse.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&parse.stderr)
    );
    let stdout = String::from_utf8_lossy(&parse.stdout);
    assert!(stdout.contains("SourceExpr"));

    let lower = Command::new(eml_bin())
        .args(["lower", "softplus(x0) + mish(x0)"])
        .output()
        .unwrap();
    assert!(
        lower.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&lower.stderr)
    );
    let stdout = String::from_utf8_lossy(&lower.stdout);
    assert!(stdout.contains("ExprStats"));
}

#[test]
fn cli_verify_and_profile_commands_run() {
    let sample_path = std::env::temp_dir().join("eml-cli-samples.json");
    fs::write(&sample_path, "[[0.2, 1.4], [0.5, 2.0]]").unwrap();

    let verify = Command::new(eml_bin())
        .args([
            "verify",
            "exp(x0) - log(x1)",
            "--samples",
            sample_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&verify.stderr)
    );
    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(stdout.contains("passed=true"));
    assert!(stdout.contains("backend=bytecode"));

    let mish_verify = Command::new(eml_bin())
        .args([
            "verify",
            "mish(x0)",
            "--samples",
            sample_path.to_str().unwrap(),
            "--relaxed",
        ])
        .output()
        .unwrap();
    assert!(
        mish_verify.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&mish_verify.stderr)
    );
    let stdout = String::from_utf8_lossy(&mish_verify.stdout);
    assert!(stdout.contains("backend=bytecode"));

    let profile = Command::new(eml_bin())
        .args(["profile", "softplus(x0) + mish(x0)", "--relaxed"])
        .output()
        .unwrap();
    assert!(
        profile.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&profile.stderr)
    );
    let stdout = String::from_utf8_lossy(&profile.stdout);
    assert!(stdout.contains("compile_total_ms="));
    assert!(stdout.contains("eval_backend=bytecode"));
    assert!(stdout.contains("eval_total_ms="));
}
