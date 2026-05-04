# Interoperability / 互操作

## 中文

P16 的目标是把 `SourceExpr` 与纯 EML `Expr` 导出为稳定、可读、可被外部工具消费的 portable graph JSON。它不是一个新的训练框架，也不是自研 runtime；它用于研究实验中的对照、反降级和后端适配。

### Portable Graph JSON

顶层结构：

```json
{
  "schema": "eml-rs.portable-graph.v1",
  "graph_kind": "source_expr",
  "root": 3,
  "nodes": [
    { "id": 0, "op": "var", "inputs": [], "attrs": { "index": 0 } }
  ]
}
```

字段含义：
- `schema`: 格式版本，当前为 `eml-rs.portable-graph.v1`。
- `graph_kind`: `source_expr` 或 `eml_expr`。
- `root`: 根节点 id。
- `nodes`: 后序节点列表，每个节点包含 `id/op/inputs/attrs`。

### SourceExpr 导出

`SourceExpr` 导出保留源算子，例如 `add/log/softplus/mish/gelu_tanh`。这条路径适合做 PyTorch/NumPy 对照，因为目标框架通常已经有对应高层算子或可直接表达的组合。

```rust
let source = eml_rs::lowering::parse_source_expr("softplus(x0) + log(x1)")?;
let json = eml_rs::portable::source_expr_to_portable_json(&source)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Expr 导出

`Expr` 导出只保留纯 EML 节点：
- `one`
- `var`
- `eml`

其中 `eml(lhs, rhs)` 的语义固定为 `exp(lhs) - ln(rhs)`，并写入节点 `attrs.formula`。这条路径适合硬件实验、统一 IR 研究，以及把纯 EML 图反降级到目标框架算子图。

```rust
let expr = eml_rs::lowering::parse_and_lower("exp(x0) - log(x1)")?;
let json = eml_rs::portable::expr_to_portable_json(&expr)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### PyTorch/NumPy 对照脚本

脚本入口：

```bash
python scripts/reference_compare.py --graph target/source-graph.json --samples target/samples.json
```

样本格式：

```json
[[0.2, 1.4], [0.5, 2.0]]
```

脚本行为：
- 始终尝试使用 NumPy 执行 portable graph。
- 如果安装了 PyTorch，则额外执行 Torch 后端并输出 `max_abs_numpy_torch`。
- 如果没有 PyTorch，`torch_available=false`，不会失败。

### 反降级到目标框架

推荐流程：
1. 研究阶段用 `SourceExpr` 或 `Expr` 保留统一语义。
2. 导出 portable graph JSON。
3. 在外部框架中把节点映射回目标算子。
4. 对 `source_expr`，优先映射为高层算子，如 `softplus/sigmoid/tanh`。
5. 对 `eml_expr`，把每个 `eml(lhs,rhs)` 映射为 `exp(lhs) - log(rhs)`。
6. 由目标框架自己的 compiler/kernel fusion 继续优化。

边界：portable graph 是交换格式，不承诺运行时性能；性能仍由目标框架、硬件后端和图优化器决定。

## English

P16 exports `SourceExpr` and pure EML `Expr` into a stable, readable portable graph JSON format. It is not a new training framework or runtime; it is an exchange layer for research comparisons, de-lowering, and backend integration.

### Portable Graph JSON

Top-level structure:

```json
{
  "schema": "eml-rs.portable-graph.v1",
  "graph_kind": "source_expr",
  "root": 3,
  "nodes": [
    { "id": 0, "op": "var", "inputs": [], "attrs": { "index": 0 } }
  ]
}
```

Fields:
- `schema`: format version, currently `eml-rs.portable-graph.v1`.
- `graph_kind`: `source_expr` or `eml_expr`.
- `root`: root node id.
- `nodes`: post-order node list; each node has `id/op/inputs/attrs`.

### SourceExpr Export

`SourceExpr` export keeps source operators such as `add/log/softplus/mish/gelu_tanh`. This path is best for PyTorch/NumPy comparisons because target frameworks usually have equivalent high-level operators or direct compositions.

```rust
let source = eml_rs::lowering::parse_source_expr("softplus(x0) + log(x1)")?;
let json = eml_rs::portable::source_expr_to_portable_json(&source)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Expr Export

`Expr` export keeps only pure EML nodes:
- `one`
- `var`
- `eml`

The semantics of `eml(lhs, rhs)` are fixed as `exp(lhs) - ln(rhs)` and recorded in `attrs.formula`. This path is intended for hardware experiments, unified IR research, and de-lowering pure EML graphs back into target framework operator graphs.

```rust
let expr = eml_rs::lowering::parse_and_lower("exp(x0) - log(x1)")?;
let json = eml_rs::portable::expr_to_portable_json(&expr)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### PyTorch/NumPy Reference Script

Script entry:

```bash
python scripts/reference_compare.py --graph target/source-graph.json --samples target/samples.json
```

Sample format:

```json
[[0.2, 1.4], [0.5, 2.0]]
```

Behavior:
- NumPy is always attempted.
- If PyTorch is installed, the script also runs Torch and reports `max_abs_numpy_torch`.
- Without PyTorch, `torch_available=false` and the script still succeeds.

### De-Lowering Into Target Frameworks

Recommended flow:
1. Keep unified semantics in `SourceExpr` or `Expr` during research.
2. Export portable graph JSON.
3. Map nodes back into target framework operators.
4. For `source_expr`, prefer high-level operators such as `softplus/sigmoid/tanh`.
5. For `eml_expr`, map every `eml(lhs,rhs)` to `exp(lhs) - log(rhs)`.
6. Let the target framework compiler/kernel fusion optimize the result.

Boundary: portable graph is an exchange format, not a performance promise. Runtime performance still depends on the target framework, hardware backend, and graph optimizer.
