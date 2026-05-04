# Versioning / 版本策略

## 中文

### 语义化版本
- `0.x`: 研究阶段，可在 minor 版本中调整 API，但必须附迁移说明。
- `1.x`: 进入稳定阶段，遵守 SemVer，破坏性改动只能出现在 major 版本。

### MSRV
- 当前 MSRV：`Rust 1.86`
- 规则：MSRV 升级必须写入变更日志和版本文档。

### 文档版本化
- 每个正式发布版本都应在 `docs/releases/<version>/` 保留对应快照。
- `README.md` 与 `docs/*.md` 的主分支内容代表最新开发线。

### 弃用流程
1. 在代码中添加 `#[deprecated(since = "...", note = "...")]`，并保留兼容实现。
2. 在 `docs/api-stability.md` 和对应使用文档中标记 deprecated，写明替代入口。
3. 至少保留一个 minor 周期。
4. 删除前在发布说明中给出替代路径。

当前代码级流程示例是 `eml_rs::api::compile_expression()`，替代入口是 `eml_rs::api::compile()`。

### API 分层
- Stable API：`compile()`, `PipelineBuilder`, `CompiledPipeline`, `BuiltinBackend`, `PipelineOptions`, `error::*`, `core::EvalPolicy`。
- Experimental API：`ir`, `bytecode`, `lowering`, `opt`, `verify`, `profiling`, `portable`, `plugin`, Rust 侧 `ffi`。
- Internal API：不建议生产代码直接依赖的实现细节；当前公开仅服务研究实验和调试。

完整规则见 `docs/api-stability.md`。

## English

### Semantic Versioning
- `0.x`: research phase; minor releases may adjust APIs, but migration notes are required.
- `1.x`: stable phase; SemVer applies and breaking changes move to major releases.

### MSRV
- Current MSRV: `Rust 1.86`
- Rule: every MSRV bump must be documented in the changelog and versioning docs.

### Documentation Versioning
- Every formal release should keep a matching snapshot under `docs/releases/<version>/`.
- `README.md` and the top-level `docs/*.md` files on the main branch describe the latest development line.

### Deprecation Flow
1. Add `#[deprecated(since = "...", note = "...")]` in code and keep a compatibility implementation.
2. Mark the item as deprecated in `docs/api-stability.md` and the relevant user docs, with a replacement entry.
3. Keep it for at least one minor cycle.
4. Provide a replacement path before removal.

The current code-level workflow example is `eml_rs::api::compile_expression()`, replaced by `eml_rs::api::compile()`.

### API Tiers
- Stable API: `compile()`, `PipelineBuilder`, `CompiledPipeline`, `BuiltinBackend`, `PipelineOptions`, `error::*`, `core::EvalPolicy`.
- Experimental API: `ir`, `bytecode`, `lowering`, `opt`, `verify`, `profiling`, `portable`, `plugin`, Rust-side `ffi`.
- Internal API: implementation details not recommended for production dependencies; public mainly for research and debugging.

See `docs/api-stability.md` for the full policy.
