# Versioning / 版本策略

## 中文

### 语义化版本
- `0.x`: 研究阶段，可在 minor 版本中调整 API，但必须附迁移说明。
- `1.x`: 进入稳定阶段，遵守 SemVer，破坏性改动只能出现在 major 版本。

### MSRV
- 当前 MSRV：`Rust 1.75`
- 规则：MSRV 升级必须写入变更日志和版本文档。

### 文档版本化
- 每个正式发布版本都应在 `docs/releases/<version>/` 保留对应快照。
- `README.md` 与 `docs/*.md` 的主分支内容代表最新开发线。

### 弃用流程
1. 在文档中标记 deprecated。
2. 至少保留一个 minor 周期。
3. 删除前在发布说明中给出替代路径。

## English

### Semantic Versioning
- `0.x`: research phase; minor releases may adjust APIs, but migration notes are required.
- `1.x`: stable phase; SemVer applies and breaking changes move to major releases.

### MSRV
- Current MSRV: `Rust 1.75`
- Rule: every MSRV bump must be documented in the changelog and versioning docs.

### Documentation Versioning
- Every formal release should keep a matching snapshot under `docs/releases/<version>/`.
- `README.md` and the top-level `docs/*.md` files on the main branch describe the latest development line.

### Deprecation Flow
1. Mark the item as deprecated in docs.
2. Keep it for at least one minor cycle.
3. Provide a replacement path before removal.
