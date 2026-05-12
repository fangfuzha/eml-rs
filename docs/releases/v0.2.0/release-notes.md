# v0.2.0 Release Notes / 发布说明

## 中文

`v0.2.0` 的目标不是扩大运行时能力面，而是把论文复现资产提升到“可审阅、可追踪、可发布前核对”的治理层，并补齐外部工具消费 portable graph 的命令行入口。

### 本阶段范围

- 冻结 `paper-basis` catalog 的 schema 与双语目录表达，确保 `docs/paper-basis-catalog.json` 成为脚本、测试、发布审阅的机器可读事实源。
- 保留并扩展代表性 witness replay，覆盖第一批基础 witness、P22 缺口补齐项，以及组合命名入口。
- 将 paper reproduction summary 升级为 schema v2，补充覆盖率、missing/partial 明细、witness provenance 与可选严格 gate 参数。
- 将符号回归研究 benchmark 升级为多任务、多 seed 的 schema v2 artifact，并把 snapping 规则与失败摘要写入文档协议。
- 增加 `eml export portable <expr> [--kind source|eml]`，让外部工具无需写 Rust 代码即可获得 portable graph JSON。
- 明确 P27 性能门禁策略：PR CI 保持格式、静态检查、测试、覆盖率、兼容性与供应链检查；运行时性能回退继续由 nightly / 手动 `bench-only` 阻断。

### API 与迁移

- Stable API 推荐路径继续是 `compile()`、`PipelineBuilder`、`CompiledPipeline`、`BuiltinBackend`、`PipelineOptions`、`error::*` 与 `core::EvalPolicy`。
- `compile_expression()` 作为弃用流程示例继续保留整个 `0.2.x` 周期；最早移除目标是 `0.3.0` 或之后版本，删除前会再次在 release notes 提醒。
- portable graph 的 Rust helper 仍归类为 Experimental API，但 `eml-rs.portable-graph.v1` schema 在 `0.2.x` 周期内作为稳定交换格式处理。

### 当前已知边界

- `repo-extension` 训练模板仍然属于工程扩展，不构成论文 Table 1 完备性声明的一部分。
- paper reproduction summary 仍以 nightly / `workflow_dispatch` 非阻断 artifact 为主；严格 gate 只在手动触发时启用。
- SR research artifact 继续保持研究面定位，不与运行时性能 gate 混合，也不作为发布阻断条件。
- 正式 GitHub Release 仍以 `v0.2.0` tag、release workflow 与 release assets 的最终发布为准。

### 发布前核对重点

- 论文复现资产可从 README、用户文档、开发者文档与 release 快照中直接发现。
- `paper-basis` catalog、paper reproduction harness、summary 脚本三者使用一致的 witness 事实源。
- SR research JSON/Markdown 结构已文档化，不依赖隐式脚本约定。
- workspace crate 版本为 `0.2.0`，并与 `Cargo.lock` 中的本地包版本一致。

## English

`v0.2.0` focuses on governance and auditability rather than expanding the runtime surface. It makes paper-reproduction assets reviewable, traceable, and release-prep friendly, and adds a CLI entry point for external tools to consume portable graph JSON.

### Scope For This Stage

- Freeze the `paper-basis` catalog schema and bilingual catalog presentation so `docs/paper-basis-catalog.json` remains the machine-readable source of truth for scripts, tests, and release review.
- Preserve and extend representative witness replay to cover the first basis witness batch, the P22 gap fills, and the composition-only named entries.
- Upgrade the paper reproduction summary to schema v2 with coverage ratio, missing/partial detail, witness provenance, and optional strict-gate parameters.
- Upgrade the symbolic-regression research benchmark to a multi-task, multi-seed schema v2 artifact and document its snapping rules plus failure summaries.
- Add `eml export portable <expr> [--kind source|eml]` so external tools can obtain portable graph JSON without writing Rust glue code.
- Record the P27 performance-gate decision: PR CI stays focused on formatting, linting, tests, coverage, compatibility, and supply-chain checks; runtime performance regressions continue to block through nightly / manual `bench-only`.

### API And Migration

- The recommended Stable API path remains `compile()`, `PipelineBuilder`, `CompiledPipeline`, `BuiltinBackend`, `PipelineOptions`, `error::*`, and `core::EvalPolicy`.
- `compile_expression()` remains available as a deprecation-flow example throughout the `0.2.x` cycle. The earliest removal target is `0.3.0` or later, with another release-notes reminder before deletion.
- Rust-side portable graph helpers remain Experimental API, but the `eml-rs.portable-graph.v1` schema is treated as a stable exchange format throughout the `0.2.x` cycle.

### Current Boundaries

- `repo-extension` training templates remain engineering extensions and are not part of any paper Table 1 completeness claim.
- The paper reproduction summary still operates primarily as a nightly / `workflow_dispatch` non-blocking artifact; the strict gate is manual-only.
- The SR research artifact remains a research track and is intentionally separate from runtime performance gates and release blockers.
- The formal GitHub Release is still defined by the final `v0.2.0` tag, release workflow, and release assets.

### Pre-release Review Focus

- Paper-reproduction assets must be directly discoverable from the README, user guide, developer guide, and release snapshot.
- The `paper-basis` catalog, paper reproduction harness, and summary script must share one witness source of truth.
- The SR research JSON/Markdown structure must be documented instead of relying on implicit script behavior.
- Workspace crate versions are `0.2.0` and match the local package versions in `Cargo.lock`.
