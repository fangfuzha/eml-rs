# v0.2.0 Verification Checklist / 发布前验证清单

## 中文

### 本地/CI 代码验证

- `cargo test --all-targets`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

### 论文复现资产验证

- `cargo test --test paper_reproduction`
- `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md`
- 如需手动严格审阅：
  - `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md --require-all-covered --require-no-missing-replayed --require-min-covered-ratio 1.0`

### SR 研究资产验证

- `python scripts/sr_research_benchmark.py --samples 41 --steps 80 --output-json target/sr-research-benchmark.json --output-md target/sr-research-benchmark.md`
- 核对 `schema = eml-rs.sr-research-benchmark.v2`、任务集、seed 集、snapping 规则与失败摘要是否产出。

### Artifact 可访问性与发布审阅

- nightly / `workflow_dispatch` 的 `paper-reproduction-summary` artifact 可下载。
- nightly / `workflow_dispatch` 的 `sr-research-benchmark` artifact 可下载。
- README、`docs/user-guide.md`、`docs/developer-guide.md`、`docs/releases/v0.2.0/` 均能指向 `paper-basis` 入口。
- 发布说明中必须明确：`repo-extension` 训练模板不属于论文基集完备性声明。

## English

### Local / CI Code Validation

- `cargo test --all-targets`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

### Paper-Reproduction Asset Validation

- `cargo test --test paper_reproduction`
- `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md`
- For optional manual strict review:
  - `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md --require-all-covered --require-no-missing-replayed --require-min-covered-ratio 1.0`

### SR Research Asset Validation

- `python scripts/sr_research_benchmark.py --samples 41 --steps 80 --output-json target/sr-research-benchmark.json --output-md target/sr-research-benchmark.md`
- Confirm that `schema = eml-rs.sr-research-benchmark.v2`, the task set, seed set, snapping rules, and failure summary are all emitted.

### Artifact Discoverability And Release Review

- The nightly / `workflow_dispatch` `paper-reproduction-summary` artifact must be downloadable.
- The nightly / `workflow_dispatch` `sr-research-benchmark` artifact must be downloadable.
- `README.md`, `docs/user-guide.md`, `docs/developer-guide.md`, and `docs/releases/v0.2.0/` must all point users toward the `paper-basis` entry points.
- Release notes must explicitly state that `repo-extension` training templates are outside any paper-basis completeness claim.
