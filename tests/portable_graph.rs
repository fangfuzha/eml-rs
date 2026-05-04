use eml_rs::ir::Expr;
use eml_rs::lowering::parse_source_expr;
use eml_rs::portable::{expr_to_portable_json, source_expr_to_portable_json};

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
