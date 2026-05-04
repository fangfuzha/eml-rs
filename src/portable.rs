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
            SourceExpr::Sqrt(inner) => self.push_unary_source("sqrt", inner),
            SourceExpr::Sigmoid(inner) => self.push_unary_source("sigmoid", inner),
            SourceExpr::Softplus(inner) => self.push_unary_source("softplus", inner),
            SourceExpr::Swish(inner) => self.push_unary_source("swish", inner),
            SourceExpr::GeluTanh(inner) => self.push_unary_source("gelu_tanh", inner),
            SourceExpr::ReluSoft(inner) => self.push_unary_source("relu_soft", inner),
            SourceExpr::Elu(lhs, rhs) => self.push_binary_source("elu", lhs, rhs),
            SourceExpr::LeakyRelu(lhs, rhs) => self.push_binary_source("leaky_relu", lhs, rhs),
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
