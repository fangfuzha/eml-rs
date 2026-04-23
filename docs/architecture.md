# Architecture / 架构设计

## 中文

### 分层总览

| 层级 | 主要模块 | 职责 |
| --- | --- | --- |
| 算法层 | `crates/eml-lowering`, `src/opt.rs` | 解析、模板构造、自动微分、源级重写、降级到纯 EML |
| 平台抽象层 | `crates/eml-core`, `src/ir.rs`, `src/bytecode.rs`, `src/verify.rs`, `src/error.rs`, `src/plugin.rs` | EML 数值语义、统一 IR、执行后端、验证、诊断、扩展点 |
| API 层 | `src/api.rs`, `src/lowering.rs`, `src/core.rs` | 高层开箱即用流程与低层兼容入口 |
| 绑定层 | `src/ffi.rs`, `include/eml_rs.h`, `examples/c_ffi_minimal` | C ABI 与非 Rust 集成边界 |
| 工具层 | `benches/`, `benchmarks/`, `scripts/`, `.github/workflows/`, `docs/` | 基准、门禁、CI/CD、文档、治理资产 |

### 依赖方向
- `eml-core` 只提供数值内核与策略，不依赖 `std`，也不依赖 parser/lowering。
- `eml-lowering` 只负责源表达式与纯 EML 降级，不依赖 runtime API、FFI、CI 工具。
- 根 crate 负责把 lowering/core/IR/bytecode/verify 串成一个研究友好的统一框架。
- FFI 只能依赖 API 或 core 边界，不能反向把 C ABI 语义渗回核心模块。

### 跨层调用规则（冻结）
- 算法层不得依赖绑定层或工具层。
- `eml-core` 与 `eml-lowering` 保持独立、可单独复用，不允许依赖根 crate。
- API 层可以编排 lower/opt/IR/bytecode/verify，但不直接内嵌 CI、benchmark 逻辑。
- 绑定层只暴露稳定、窄接口，不直接暴露内部 IR 结构体内存布局。
- 工具层只能通过公开 API 或 CLI/脚本入口消费项目，不修改核心语义。

### API 分层

| 层级 | 入口 | 用途 |
| --- | --- | --- |
| 高层 API | `eml_rs::api::{compile, PipelineBuilder}` | 一次性完成 parse/opt/lower/compile/eval/verify |
| 低层 API | `core`, `lowering`, `ir`, `bytecode`, `verify`, `opt` | 研究场景下对单个环节精细控制 |
| 扩展点 | `plugin::{SourcePass, ExprPass, ExecutionBackend, PipelineObserver}` | 自定义 pass、执行后端、观测钩子 |

### 错误处理体系
- Rust 侧统一使用 `EmlError`。
- 每个错误都映射到稳定的 `EmlErrorCode` 与 `EmlDiagnostic`。
- 诊断分三类：`semantic`、`execution`、`compile`。
- 约束：禁止无意义 `panic` 作为用户输入错误处理手段；用户可恢复错误必须走 `Result`。

### 插件化扩展点
- `SourcePass`: 在 lowering 前改写 `SourceExpr`。
- `ExprPass`: 在 lowering 后改写纯 EML `Expr`。
- `ExecutionBackend`: 给实验型执行器留入口，不强迫进入 builtin tree/RPN/bytecode。
- `PipelineObserver`: 用于统计节点数、阶段耗时、字节码大小等观测信息。

### 安全约束
- 根 crate 启用 `#![deny(unsafe_op_in_unsafe_fn)]`。
- `eml-core` 与 `eml-lowering` 启用 `#![forbid(unsafe_code)]`。
- 当前 `unsafe` 只允许存在于 FFI 指针边界，且必须带显式 `// SAFETY:` 注释。
- 线程安全策略：插件 trait 统一要求 `Send + Sync`，避免并行基准或未来多线程编排时出现隐式共享问题。

### 稳定性承诺
- `1.0` 前允许在 minor 版本中调整 API，但必须更新迁移说明。
- `1.0` 后遵守 SemVer。
- MSRV 冻结为 `Rust 1.75`，如需升级必须在版本文档中显式说明。

## English

### Layer Overview

| Layer | Main modules | Responsibility |
| --- | --- | --- |
| Algorithm layer | `crates/eml-lowering`, `src/opt.rs` | Parsing, templates, autodiff, source rewrites, lowering into pure EML |
| Platform abstraction | `crates/eml-core`, `src/ir.rs`, `src/bytecode.rs`, `src/verify.rs`, `src/error.rs`, `src/plugin.rs` | Numeric EML semantics, unified IR, execution backends, verification, diagnostics, extension points |
| API layer | `src/api.rs`, `src/lowering.rs`, `src/core.rs` | High-level orchestration plus low-level compatibility entry points |
| Binding layer | `src/ffi.rs`, `include/eml_rs.h`, `examples/c_ffi_minimal` | C ABI and non-Rust integration boundary |
| Tooling layer | `benches/`, `benchmarks/`, `scripts/`, `.github/workflows/`, `docs/` | Benchmarks, gates, CI/CD, docs, governance assets |

### Dependency Direction
- `eml-core` owns numeric kernels and policies, stays `no_std`, and does not depend on parsing/lowering.
- `eml-lowering` owns source expressions and pure-EML lowering, and does not depend on runtime APIs, FFI, or CI tooling.
- The root crate composes lowering/core/IR/bytecode/verify into a research-oriented unified framework.
- FFI may depend on API/core boundaries, but the C ABI must not leak back into core semantics.

### Cross-layer Rules
- The algorithm layer must not depend on bindings or tooling.
- `eml-core` and `eml-lowering` stay independently reusable and may not depend on the root crate.
- The API layer may orchestrate lower/opt/IR/bytecode/verify, but it must not embed CI or benchmark logic.
- The binding layer exposes narrow stable interfaces and must not expose internal IR layout as ABI.
- The tooling layer consumes the project through public APIs or scripts and must not redefine semantics.

### API Layers

| Layer | Entry point | Purpose |
| --- | --- | --- |
| High-level API | `eml_rs::api::{compile, PipelineBuilder}` | Parse/optimize/lower/compile/evaluate/verify in one place |
| Low-level API | `core`, `lowering`, `ir`, `bytecode`, `verify`, `opt` | Fine-grained control for research workflows |
| Extension points | `plugin::{SourcePass, ExprPass, ExecutionBackend, PipelineObserver}` | Custom passes, backends, and observability hooks |

### Error Handling
- Rust-side APIs use a unified `EmlError`.
- Every error maps to a stable `EmlErrorCode` and `EmlDiagnostic`.
- Diagnostics are grouped into `semantic`, `execution`, and `compile`.
- Rule: user-recoverable failures must use `Result`; meaningless `panic` is not an input-validation strategy.

### Extension Points
- `SourcePass`: rewrites `SourceExpr` before lowering.
- `ExprPass`: rewrites pure EML `Expr` after lowering.
- `ExecutionBackend`: experimental executor hook beyond builtin tree/RPN/bytecode.
- `PipelineObserver`: stage-level instrumentation for node counts, timings, and bytecode size.

### Security Constraints
- The root crate enables `#![deny(unsafe_op_in_unsafe_fn)]`.
- `eml-core` and `eml-lowering` enable `#![forbid(unsafe_code)]`.
- `unsafe` is only allowed at the FFI pointer boundary and must carry explicit `// SAFETY:` comments.
- Plugin traits require `Send + Sync` to avoid implicit sharing hazards in future parallel orchestration.

### Stability Commitments
- Before `1.0`, minor releases may adjust APIs, but migration notes must be updated.
- After `1.0`, SemVer applies.
- MSRV is frozen at `Rust 1.75`; any bump must be documented in the versioning policy.
