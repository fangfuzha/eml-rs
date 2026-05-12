use eml_rs::ir::Expr;
use eml_rs::lowering::parse_source_expr;
use eml_rs::portable::{
    expr_to_portable_json, source_expr_to_portable_json, validate_portable_graph,
    validate_portable_json,
};
use serde_json::json;
use std::fs;
use std::process::Command;

fn ops(value: &serde_json::Value) -> Vec<String> {
    value["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["op"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn source_expr_exports_portable_graph_json() {
    let source = parse_source_expr("softplus(x0) + log(x1)").unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&source_expr_to_portable_json(&source).unwrap()).unwrap();

    assert_eq!(value["schema"], "eml-rs.portable-graph.v1");
    validate_portable_graph(&value).unwrap();
    assert_eq!(value["graph_kind"], "source_expr");
    assert_eq!(value["root"], value["nodes"].as_array().unwrap().len() - 1);
    let ops = ops(&value);
    assert!(ops.contains(&"var".to_string()));
    assert!(ops.contains(&"softplus".to_string()));
    assert!(ops.contains(&"log".to_string()));
    assert!(ops.contains(&"add".to_string()));
}

#[test]
fn expr_exports_eml_semantics() {
    let expr = Expr::eml(Expr::exp(Expr::var(0)), Expr::ln(Expr::var(1)));
    let value: serde_json::Value =
        serde_json::from_str(&expr_to_portable_json(&expr).unwrap()).unwrap();

    validate_portable_graph(&value).unwrap();
    assert_eq!(value["graph_kind"], "eml_expr");
    let ops = ops(&value);
    assert!(ops.contains(&"one".to_string()));
    assert!(ops.contains(&"var".to_string()));
    assert!(ops.iter().filter(|op| *op == "eml").count() >= 1);

    let root = value["root"].as_u64().unwrap() as usize;
    let root_node = &value["nodes"].as_array().unwrap()[root];
    assert_eq!(root_node["op"], "eml");
    assert_eq!(root_node["inputs"].as_array().unwrap().len(), 2);
}

#[test]
fn portable_graph_validator_rejects_invalid_shape() {
    let bad = json!({
        "schema": "eml-rs.portable-graph.v1",
        "graph_kind": "source_expr",
        "root": 1,
        "nodes": [
            { "id": 0, "op": "var", "inputs": [], "attrs": { "index": 0 } },
            { "id": 1, "op": "hypot", "inputs": [0], "attrs": {} }
        ]
    });
    let err = validate_portable_graph(&bad).unwrap_err();
    assert!(err.contains("expected 2 inputs"));

    let json = source_expr_to_portable_json(&parse_source_expr("hypot(x0, x1)").unwrap()).unwrap();
    validate_portable_json(&json).unwrap();
}

fn python_command() -> &'static str {
    if Command::new("python").arg("--version").output().is_ok() {
        "python"
    } else {
        "python3"
    }
}

#[test]
fn reference_compare_script_supports_p22_portable_ops() {
    let source = parse_source_expr("asinh(x0) + acosh(x1) + atanh(x2) + hypot(x0, x2)").unwrap();
    let graph = source_expr_to_portable_json(&source).unwrap();
    let temp = std::env::temp_dir();
    let graph_path = temp.join("eml-portable-p22-graph.json");
    let samples_path = temp.join("eml-portable-p22-samples.json");
    fs::write(&graph_path, graph).unwrap();
    fs::write(&samples_path, "[[0.5, 1.5, 0.25], [0.25, 2.0, 0.1]]").unwrap();

    let script = format!(
        "{}/scripts/reference_compare.py",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let output = Command::new(python_command())
        .args([
            script.as_str(),
            "--graph",
            graph_path.to_str().unwrap(),
            "--samples",
            samples_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"passed\": true"));
}
