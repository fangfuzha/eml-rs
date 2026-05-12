//! Portable graph JSON 导出工具。
//!
//! 该模块属于根 crate，而不是 `eml-lowering`，因为 JSON 依赖 `serde_json`，
//! 不应进入 `no_std + alloc` 的 parser/lowering 分层。

use serde_json::{json, Value};

use crate::ir::Expr;
use crate::lowering::SourceExpr;

const SCHEMA: &str = "eml-rs.portable-graph.v1";

/// 将 `SourceExpr` 导出为 portable graph JSON value。
///
/// 节点使用后序编号，`root` 总是指向根节点。源表达式图保留源算子，
/// 适合作为 PyTorch/NumPy 等外部框架对照脚本的输入。
///
/// # 示例
///
/// ```rust
/// let source = eml_rs::lowering::parse_source_expr("softplus(x0) + log(x1)")?;
/// let graph = eml_rs::portable::source_expr_to_portable_graph(&source);
/// assert_eq!(graph["graph_kind"], "source_expr");
/// # Ok::<(), eml_rs::EmlError>(())
/// ```
pub fn source_expr_to_portable_graph(expr: &SourceExpr) -> Value {
    let mut builder = Builder::default();
    let root = builder.push_source(expr);
    builder.finish("source_expr", root)
}

/// 将 `SourceExpr` 导出为 pretty JSON 字符串。
///
/// # 示例
///
/// ```rust
/// let source = eml_rs::lowering::parse_source_expr("exp(x0)").unwrap();
/// let json = eml_rs::portable::source_expr_to_portable_json(&source)?;
/// assert!(json.contains("source_expr"));
/// # Ok::<(), serde_json::Error>(())
/// ```
pub fn source_expr_to_portable_json(expr: &SourceExpr) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&source_expr_to_portable_graph(expr))
}

/// 将运行时 `Expr` 导出为 portable graph JSON value。
///
/// `Expr` 只包含 `one`、`var`、`eml` 三种节点，因此导出结果保留纯 EML
/// 语义，便于硬件或外部图框架做后续反降级。
///
/// # 示例
///
/// ```rust
/// let expr = eml_rs::ir::Expr::eml(eml_rs::ir::Expr::var(0), eml_rs::ir::Expr::one());
/// let graph = eml_rs::portable::expr_to_portable_graph(&expr);
/// assert_eq!(graph["graph_kind"], "eml_expr");
/// ```
pub fn expr_to_portable_graph(expr: &Expr) -> Value {
    let mut builder = Builder::default();
    let root = builder.push_expr(expr);
    builder.finish("eml_expr", root)
}

/// 将运行时 `Expr` 导出为 pretty JSON 字符串。
///
/// # 示例
///
/// ```rust
/// let expr = eml_rs::ir::Expr::exp(eml_rs::ir::Expr::var(0));
/// let json = eml_rs::portable::expr_to_portable_json(&expr)?;
/// assert!(json.contains("\"eml\""));
/// # Ok::<(), serde_json::Error>(())
/// ```
pub fn expr_to_portable_json(expr: &Expr) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&expr_to_portable_graph(expr))
}

/// 校验 portable graph JSON value 的基础结构。
///
/// 校验范围保持轻量：schema、graph kind、节点 id、root、输入索引、attrs
/// 类型与当前支持的 op arity。它不执行数值语义验证。
pub fn validate_portable_graph(graph: &Value) -> Result<(), String> {
    let schema = graph
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing string field: schema".to_string())?;
    if schema != SCHEMA {
        return Err(format!("unsupported schema: {schema}"));
    }

    let graph_kind = graph
        .get("graph_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing string field: graph_kind".to_string())?;
    if graph_kind != "source_expr" && graph_kind != "eml_expr" {
        return Err(format!("unsupported graph_kind: {graph_kind}"));
    }

    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: nodes".to_string())?;
    if nodes.is_empty() {
        return Err("nodes must not be empty".to_string());
    }

    let root = graph
        .get("root")
        .and_then(Value::as_u64)
        .ok_or_else(|| "missing unsigned integer field: root".to_string())? as usize;
    if root >= nodes.len() {
        return Err(format!("root index out of bounds: {root}"));
    }

    for (expected_id, node) in nodes.iter().enumerate() {
        let id = node
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("node {expected_id} missing unsigned integer field: id"))?
            as usize;
        if id != expected_id {
            return Err(format!(
                "node id mismatch: expected {expected_id}, got {id}"
            ));
        }

        let op = node
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("node {id} missing string field: op"))?;
        let attrs = node
            .get("attrs")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("node {id} missing object field: attrs"))?;
        validate_attrs(op, attrs)?;

        let inputs = node
            .get("inputs")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("node {id} missing array field: inputs"))?;
        let expected_arity = op_arity(graph_kind, op)
            .ok_or_else(|| format!("unsupported op for {graph_kind}: {op}"))?;
        if inputs.len() != expected_arity {
            return Err(format!(
                "node {id} op {op} expected {expected_arity} inputs, got {}",
                inputs.len()
            ));
        }
        for input in inputs {
            let input_id = input
                .as_u64()
                .ok_or_else(|| format!("node {id} has non-integer input"))?
                as usize;
            if input_id >= expected_id {
                return Err(format!(
                    "node {id} input {input_id} must refer to an earlier node"
                ));
            }
        }
    }

    Ok(())
}

/// 解析并校验 portable graph JSON 字符串。
pub fn validate_portable_json(input: &str) -> Result<(), String> {
    let graph: Value = serde_json::from_str(input).map_err(|err| err.to_string())?;
    validate_portable_graph(&graph)
}

#[derive(Default)]
struct Builder {
    nodes: Vec<Value>,
}

impl Builder {
    fn finish(self, graph_kind: &str, root: usize) -> Value {
        json!({
            "schema": SCHEMA,
            "graph_kind": graph_kind,
            "root": root,
            "nodes": self.nodes,
        })
    }

    fn push_node(&mut self, op: &str, inputs: Vec<usize>, attrs: Value) -> usize {
        let id = self.nodes.len();
        self.nodes.push(json!({
            "id": id,
            "op": op,
            "inputs": inputs,
            "attrs": attrs,
        }));
        id
    }

    fn push_leaf(&mut self, op: &str, attrs: Value) -> usize {
        self.push_node(op, Vec::new(), attrs)
    }

    fn push_unary_source(&mut self, op: &str, inner: &SourceExpr) -> usize {
        let input = self.push_source(inner);
        self.push_node(op, vec![input], json!({}))
    }

    fn push_binary_source(&mut self, op: &str, lhs: &SourceExpr, rhs: &SourceExpr) -> usize {
        let lhs = self.push_source(lhs);
        let rhs = self.push_source(rhs);
        self.push_node(op, vec![lhs, rhs], json!({}))
    }

    fn push_source(&mut self, expr: &SourceExpr) -> usize {
        match expr {
            SourceExpr::Var(index) => self.push_leaf("var", json!({ "index": index })),
            SourceExpr::Int(value) => self.push_leaf("int", json!({ "value": value })),
            SourceExpr::Rational(numerator, denominator) => self.push_leaf(
                "rational",
                json!({ "numerator": numerator, "denominator": denominator }),
            ),
            SourceExpr::ConstE => self.push_leaf("const_e", json!({})),
            SourceExpr::ConstI => self.push_leaf("const_i", json!({})),
            SourceExpr::ConstPi => self.push_leaf("const_pi", json!({})),
            SourceExpr::Neg(inner) => self.push_unary_source("neg", inner),
            SourceExpr::Add(lhs, rhs) => self.push_binary_source("add", lhs, rhs),
            SourceExpr::Sub(lhs, rhs) => self.push_binary_source("sub", lhs, rhs),
            SourceExpr::Mul(lhs, rhs) => self.push_binary_source("mul", lhs, rhs),
            SourceExpr::Div(lhs, rhs) => self.push_binary_source("div", lhs, rhs),
            SourceExpr::Pow(lhs, rhs) => self.push_binary_source("pow", lhs, rhs),
            SourceExpr::Exp(inner) => self.push_unary_source("exp", inner),
            SourceExpr::Log(inner) => self.push_unary_source("log", inner),
            SourceExpr::Sin(inner) => self.push_unary_source("sin", inner),
            SourceExpr::Cos(inner) => self.push_unary_source("cos", inner),
            SourceExpr::Tan(inner) => self.push_unary_source("tan", inner),
            SourceExpr::Sinh(inner) => self.push_unary_source("sinh", inner),
            SourceExpr::Cosh(inner) => self.push_unary_source("cosh", inner),
            SourceExpr::Tanh(inner) => self.push_unary_source("tanh", inner),
            SourceExpr::Asin(inner) => self.push_unary_source("asin", inner),
            SourceExpr::Acos(inner) => self.push_unary_source("acos", inner),
            SourceExpr::Atan(inner) => self.push_unary_source("atan", inner),
            SourceExpr::Asinh(inner) => self.push_unary_source("asinh", inner),
            SourceExpr::Acosh(inner) => self.push_unary_source("acosh", inner),
            SourceExpr::Atanh(inner) => self.push_unary_source("atanh", inner),
            SourceExpr::Sqrt(inner) => self.push_unary_source("sqrt", inner),
            SourceExpr::Sigmoid(inner) => self.push_unary_source("sigmoid", inner),
            SourceExpr::Softplus(inner) => self.push_unary_source("softplus", inner),
            SourceExpr::Swish(inner) => self.push_unary_source("swish", inner),
            SourceExpr::GeluTanh(inner) => self.push_unary_source("gelu_tanh", inner),
            SourceExpr::ReluSoft(inner) => self.push_unary_source("relu_soft", inner),
            SourceExpr::Elu(lhs, rhs) => self.push_binary_source("elu", lhs, rhs),
            SourceExpr::LeakyRelu(lhs, rhs) => self.push_binary_source("leaky_relu", lhs, rhs),
            SourceExpr::Hypot(lhs, rhs) => self.push_binary_source("hypot", lhs, rhs),
            SourceExpr::Softsign(inner) => self.push_unary_source("softsign", inner),
            SourceExpr::Mish(inner) => self.push_unary_source("mish", inner),
        }
    }

    fn push_expr(&mut self, expr: &Expr) -> usize {
        match expr {
            Expr::One => self.push_leaf("one", json!({})),
            Expr::Var(index) => self.push_leaf("var", json!({ "index": index })),
            Expr::Eml(lhs, rhs) => {
                let lhs = self.push_expr(lhs);
                let rhs = self.push_expr(rhs);
                self.push_node(
                    "eml",
                    vec![lhs, rhs],
                    json!({ "formula": "exp(lhs)-ln(rhs)" }),
                )
            }
        }
    }
}

fn op_arity(graph_kind: &str, op: &str) -> Option<usize> {
    match graph_kind {
        "eml_expr" => match op {
            "one" | "var" => Some(0),
            "eml" => Some(2),
            _ => None,
        },
        "source_expr" => match op {
            "var" | "int" | "rational" | "const_e" | "const_i" | "const_pi" => Some(0),
            "neg" | "exp" | "log" | "sin" | "cos" | "tan" | "sinh" | "cosh" | "tanh" | "asin"
            | "acos" | "atan" | "asinh" | "acosh" | "atanh" | "sqrt" | "sigmoid" | "softplus"
            | "swish" | "gelu_tanh" | "relu_soft" | "softsign" | "mish" => Some(1),
            "add" | "sub" | "mul" | "div" | "pow" | "elu" | "leaky_relu" | "hypot" => Some(2),
            _ => None,
        },
        _ => None,
    }
}

fn validate_attrs(op: &str, attrs: &serde_json::Map<String, Value>) -> Result<(), String> {
    match op {
        "var" => {
            if !attrs.get("index").is_some_and(Value::is_u64) {
                return Err("var attrs.index must be an unsigned integer".to_string());
            }
        }
        "int" => {
            if !attrs.get("value").is_some_and(Value::is_i64) {
                return Err("int attrs.value must be an integer".to_string());
            }
        }
        "rational" => {
            let numerator_ok = attrs.get("numerator").is_some_and(Value::is_i64);
            let denominator = attrs.get("denominator").and_then(Value::as_i64);
            if !numerator_ok || denominator.is_none() {
                return Err(
                    "rational attrs.numerator and attrs.denominator must be integers".to_string(),
                );
            }
            if denominator == Some(0) {
                return Err("rational attrs.denominator must not be zero".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}
