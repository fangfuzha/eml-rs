# v0.2.0 Draft Release Notes / 发布说明草案

## 中文

`v0.2.0` 的当前草案目标不是扩大运行时能力面，而是把论文复现资产提升到“可审阅、可追踪、可发布前核对”的治理层。

### 本阶段范围

- 冻结 `paper-basis` catalog 的 schema 与双语目录表达，确保 `docs/paper-basis-catalog.json` 成为脚本、测试、发布审阅的机器可读事实源。
- 保留并扩展代表性 witness replay，覆盖第一批基础 witness、P22 缺口补齐项，以及组合命名入口。
- 将 paper reproduction summary 升级为 schema v2，补充覆盖率、missing/partial 明细、witness provenance 与可选严格 gate 参数。
- 将符号回归研究 benchmark 升级为多任务、多 seed 的 schema v2 artifact，并把 snapping 规则与失败摘要写入文档协议。

### 当前已知边界

- `repo-extension` 训练模板仍然属于工程扩展，不构成论文 Table 1 完备性声明的一部分。
- paper reproduction summary 仍以 nightly / `workflow_dispatch` 非阻断 artifact 为主；严格 gate 只在手动触发时启用。
- SR research artifact 继续保持研究面定位，不与运行时性能 gate 混合，也不作为发布阻断条件。
- `v0.2.0` 当前仍是文档和资产准备阶段；最终是否形成正式 release，取决于后续 artifact 历史、release 检查结果和维护者决策。

### 发布前核对重点

- 论文复现资产可从 README、用户文档、开发者文档与 release 草案中直接发现。
- `paper-basis` catalog、paper reproduction harness、summary 脚本三者使用一致的 witness 事实源。
- SR research JSON/Markdown 结构已文档化，不依赖隐式脚本约定。

## English

The current `v0.2.0` draft focuses on governance and auditability rather than expanding the runtime surface. The goal is to make paper-reproduction assets reviewable, traceable, and release-prep friendly.

### Scope For This Stage

- Freeze the `paper-basis` catalog schema and bilingual catalog presentation so `docs/paper-basis-catalog.json` remains the machine-readable source of truth for scripts, tests, and release review.
- Preserve and extend representative witness replay to cover the first basis witness batch, the P22 gap fills, and the composition-only named entries.
- Upgrade the paper reproduction summary to schema v2 with coverage ratio, missing/partial detail, witness provenance, and optional strict-gate parameters.
- Upgrade the symbolic-regression research benchmark to a multi-task, multi-seed schema v2 artifact and document its snapping rules plus failure summaries.

### Current Boundaries

- `repo-extension` training templates remain engineering extensions and are not part of any paper Table 1 completeness claim.
- The paper reproduction summary still operates primarily as a nightly / `workflow_dispatch` non-blocking artifact; the strict gate is manual-only.
- The SR research artifact remains a research track and is intentionally separate from runtime performance gates and release blockers.
- `v0.2.0` is still in the release-preparation phase; whether it becomes a formal release later depends on artifact history, release-check results, and maintainer judgment.

### Pre-release Review Focus

- Paper-reproduction assets must be directly discoverable from the README, user guide, developer guide, and release draft.
- The `paper-basis` catalog, paper reproduction harness, and summary script must share one witness source of truth.
- The SR research JSON/Markdown structure must be documented instead of relying on implicit script behavior.
