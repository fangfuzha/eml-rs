# CLI / 命令行工具

## 中文

`eml` CLI 是研究实验入口，不替代库 API。它用于快速检查一条表达式在 `parse -> optimize -> lower -> compile -> verify/profile` 流程中的结构、统计与数值行为。

### 构建

```bash
cargo build --bin eml
```

### `eml parse`

把源表达式解析为 `SourceExpr`，适合排查前端语法和变量编号。

```bash
cargo run --bin eml -- parse "exp(x0) - log(x1)"
```

输出包含：
- `SourceExpr`: 解析后的源 AST。
- `source_nodes`: 源表达式节点数。

### `eml lower`

把源表达式优化后降级为纯 EML runtime IR，并输出 IR 统计。

```bash
cargo run --bin eml -- lower "softplus(x0) + sigmoid(x0)"
```

输出包含：
- `Expr`: 降级后的 EML IR。
- `ExprStats`: 节点数、深度、公共子树统计。
- `source_nodes` / `optimized_source_nodes`: lowering 前后的源表达式规模。

### `eml verify`

读取实数样本 JSON，把样本转为复数输入，并分别验证 Tree、RPN、Bytecode 三个后端是否与优化后的源表达式参考值一致。默认使用 strict 策略；当纯 EML 展开存在可去奇点中间值时，可以加 `--relaxed`。

```bash
printf '[[0.2, 1.4], [0.5, 2.0]]' > /tmp/eml-samples.json
cargo run --bin eml -- verify "exp(x0) - log(x1)" --samples /tmp/eml-samples.json --tolerance 1e-8
cargo run --bin eml -- verify "mish(x0)" --samples /tmp/eml-samples.json --relaxed
```

输出包含：
- `backend=tree|rpn|bytecode`: 每个后端的独立验证报告。
- `passed=true|false`: 三个后端是否全部通过。
- `max_abs_error`: 当前后端相对参考表达式的最大绝对误差。

### `eml profile`

输出编译阶段耗时，并用样本对 Tree、RPN、Bytecode 做批量 eval 计时。若某个后端在给定样本上触发域错误，CLI 会输出 `eval_error`，但不会让编译 profile 本身失败。对 `mish`、`gelu` 等展开较深的模板，研究对照时通常应先用 `--relaxed` 跑通，再按需切回 strict 查域问题。

```bash
cargo run --bin eml -- profile "exp(x0) - log(x1)" --sample-count 32
cargo run --bin eml -- profile "exp(x0) - log(x1)" --samples /tmp/eml-samples.json
cargo run --bin eml -- profile "softplus(x0) + mish(x0)" --relaxed
```

输出包含：
- `parse_ms` / `simplify_ms` / `lowering_ms` / `rpn_build_ms` / `bytecode_build_ms`。
- `eval_backend=tree|rpn|bytecode`。
- `eval_total_ms` / `eval_per_sample_us` / `eval_samples`。

## English

The `eml` CLI is a research-time entry point, not a replacement for the Rust API. It helps inspect one expression through `parse -> optimize -> lower -> compile -> verify/profile`.

### Build

```bash
cargo build --bin eml
```

### `eml parse`

Parse an infix expression into `SourceExpr`.

```bash
cargo run --bin eml -- parse "exp(x0) - log(x1)"
```

The output includes:
- `SourceExpr`: parsed source AST.
- `source_nodes`: source expression node count.

### `eml lower`

Optimize and lower a source expression into runtime EML IR.

```bash
cargo run --bin eml -- lower "softplus(x0) + sigmoid(x0)"
```

The output includes:
- `Expr`: lowered EML IR.
- `ExprStats`: node count, depth, and subtree statistics.
- `source_nodes` / `optimized_source_nodes`: source size before and after optimization.

### `eml verify`

Read real-valued sample JSON, convert samples to complex inputs, and verify Tree, RPN, and Bytecode against the optimized source expression reference. The default mode is strict; use `--relaxed` when a pure EML expansion has removable intermediate singularities.

```bash
printf '[[0.2, 1.4], [0.5, 2.0]]' > /tmp/eml-samples.json
cargo run --bin eml -- verify "exp(x0) - log(x1)" --samples /tmp/eml-samples.json --tolerance 1e-8
cargo run --bin eml -- verify "mish(x0)" --samples /tmp/eml-samples.json --relaxed
```

The output includes:
- `backend=tree|rpn|bytecode`: per-backend verification report.
- `passed=true|false`: whether all three backends passed.
- `max_abs_error`: maximum absolute error against the source reference.

### `eml profile`

Print compile-stage timings and batch eval timings for Tree, RPN, and Bytecode. If a backend hits a domain error on the chosen samples, the CLI prints `eval_error` for that backend while keeping compile profiling usable. For deeper templates such as `mish` and `gelu`, start with `--relaxed` for research comparisons, then switch back to strict mode when you need domain auditing.

```bash
cargo run --bin eml -- profile "exp(x0) - log(x1)" --sample-count 32
cargo run --bin eml -- profile "exp(x0) - log(x1)" --samples /tmp/eml-samples.json
cargo run --bin eml -- profile "softplus(x0) + mish(x0)" --relaxed
```

The output includes:
- `parse_ms` / `simplify_ms` / `lowering_ms` / `rpn_build_ms` / `bytecode_build_ms`.
- `eval_backend=tree|rpn|bytecode`.
- `eval_total_ms` / `eval_per_sample_us` / `eval_samples`.
