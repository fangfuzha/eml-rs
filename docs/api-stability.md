# API Stability / API 稳定性

## 中文

本文定义 `eml-rs` 在 `0.x` 阶段的 public API 分层。目标不是过早冻结所有接口，而是让研究实验用户知道哪些入口可以长期依赖，哪些入口可能随 IR/优化实验调整。

### Stable API

稳定入口优先保证语义和迁移路径。`0.x` 阶段仍允许 minor 版本调整，但必须给迁移说明。

| 入口 | 用途 | 稳定性 |
| --- | --- | --- |
| `eml_rs::api::compile()` | 一行编译表达式，推荐默认入口 | Stable API |
| `eml_rs::api::PipelineBuilder` | 可配置 pipeline，支持 options/pass/observer | Stable API |
| `eml_rs::api::CompiledPipeline` | 编译产物，提供 eval/report/verify/profile | Stable API |
| `eml_rs::api::BuiltinBackend` | Tree/RPN/Bytecode 后端枚举 | Stable API |
| `eml_rs::api::PipelineOptions` | 编译和执行策略配置 | Stable API |
| `eml_rs::error::*` | Rust 错误类型、错误码、诊断结构 | Stable API |
| `eml_rs::core::EvalPolicy` | 复对数分支和特殊值策略 | Stable API |

### Experimental API

实验入口服务研究场景，允许更快演进。使用者应固定 crate 版本，并在升级时阅读 release notes。

| 入口 | 用途 | 风险 |
| --- | --- | --- |
| `eml_rs::ir::*` | 运行时 EML IR、RPN、统计 | IR 结构可能随优化器调整 |
| `eml_rs::bytecode::*` | 寄存器字节码和指令集 | 指令布局可能变化 |
| `eml_rs::lowering::*` | parser/lowering/autodiff/template re-export | 模板和 lowering 近似可能变化 |
| `eml_rs::opt::*` | 源级重写、代价模型 | 规则和 cost 权重会继续实验 |
| `eml_rs::verify::*` | 数值对照工具 | 报告字段可扩展 |
| `eml_rs::profiling::*` | compile/eval/verify 指标 | 指标字段可扩展 |
| `eml_rs::plugin::*` | pass/backend/observer 扩展点 | trait 可能补方法 |
| `eml_rs::ffi::*` | Rust 侧 FFI 类型定义 | C ABI 以 `include/eml_rs.h` 为准 |

### Internal API

内部实现不承诺直接稳定。当前 crate 仍公开部分模块是为了研究实验和基准调试，但生产使用应优先走 Stable API。若必须依赖 Experimental API，请把版本锁定到精确 patch 版本，并保留升级测试。

### Deprecated API

弃用流程必须同时覆盖代码、文档和发布说明。

1. 代码上加 `#[deprecated(since = "...", note = "...")]`，保留兼容实现。
2. 文档中写明替代入口和迁移方式。
3. 至少保留一个 minor 周期。
4. 删除前在 release notes 里再次提示。

当前流程示例：

```rust
#[allow(deprecated)]
let pipeline = eml_rs::api::compile_expression("exp(x0)")?;
let replacement = eml_rs::api::compile("exp(x0)")?;
# let _ = (pipeline, replacement);
# Ok::<(), eml_rs::EmlError>(())
```

`compile_expression()` 只是 `compile()` 的兼容别名，用来验证弃用流程；新代码应直接调用 `compile()`。

## English

This document defines the public API tiers for the `0.x` phase. The goal is not to freeze every research surface too early, but to make it clear which entry points users can depend on and which ones may evolve with IR and optimization experiments.

### Stable API

Stable entries prioritize semantic continuity and migration paths. During `0.x`, minor releases may still adjust APIs, but migration notes are required.

| Entry | Purpose | Stability |
| --- | --- | --- |
| `eml_rs::api::compile()` | One-line expression compilation; recommended default entry | Stable API |
| `eml_rs::api::PipelineBuilder` | Configurable pipeline with options/passes/observers | Stable API |
| `eml_rs::api::CompiledPipeline` | Compiled artifact for eval/report/verify/profile | Stable API |
| `eml_rs::api::BuiltinBackend` | Tree/RPN/Bytecode backend selector | Stable API |
| `eml_rs::api::PipelineOptions` | Compile and evaluation policy options | Stable API |
| `eml_rs::error::*` | Rust error types, codes, and diagnostics | Stable API |
| `eml_rs::core::EvalPolicy` | Complex-log branch and special-value policy | Stable API |

### Experimental API

Experimental entries serve research workflows and may evolve faster. Pin the crate version and read release notes before upgrading.

| Entry | Purpose | Risk |
| --- | --- | --- |
| `eml_rs::ir::*` | Runtime EML IR, RPN, and stats | IR shape may change with optimizers |
| `eml_rs::bytecode::*` | Register bytecode and instruction set | Instruction layout may change |
| `eml_rs::lowering::*` | Parser/lowering/autodiff/template re-export | Templates and approximations may change |
| `eml_rs::opt::*` | Source rewrites and cost model | Rules and weights are still experimental |
| `eml_rs::verify::*` | Numerical comparison helpers | Report fields may expand |
| `eml_rs::profiling::*` | Compile/eval/verify metrics | Metric fields may expand |
| `eml_rs::plugin::*` | Pass/backend/observer extension points | Traits may gain methods |
| `eml_rs::ffi::*` | Rust-side FFI type definitions | The C ABI is defined by `include/eml_rs.h` |

### Internal API

Internal implementation details are not directly stabilized. Some modules remain public for research and benchmark debugging, but production users should prefer the Stable API. If you depend on Experimental API, pin to an exact patch version and keep upgrade tests.

### Deprecated API

Deprecation must cover code, docs, and release notes.

1. Add `#[deprecated(since = "...", note = "...")]` in code and keep a compatibility implementation.
2. Document the replacement entry and migration path.
3. Keep the deprecated item for at least one minor cycle.
4. Repeat the removal notice in release notes before deleting it.

Current workflow example:

```rust
#[allow(deprecated)]
let pipeline = eml_rs::api::compile_expression("exp(x0)")?;
let replacement = eml_rs::api::compile("exp(x0)")?;
# let _ = (pipeline, replacement);
# Ok::<(), eml_rs::EmlError>(())
```

`compile_expression()` is only a compatibility alias for `compile()` to exercise the deprecation workflow. New code should call `compile()` directly.
