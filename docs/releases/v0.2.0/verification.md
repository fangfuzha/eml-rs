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
- 下载 artifact 后运行 `python scripts/nightly_artifact_audit.py --artifact-root <download-dir> --require-paper-all-covered --require-sr-non-blocking --output-json target/nightly-artifact-audit.json --output-md target/nightly-artifact-audit.md`，核对 v2 schema、paper 覆盖率与 SR 非阻断策略。
- README、`docs/user-guide.md`、`docs/developer-guide.md`、`docs/releases/v0.2.0/` 均能指向 `paper-basis` 入口。
- 发布说明中必须明确：`repo-extension` 训练模板不属于论文基集完备性声明。

### 2026-05-12 本地验证记录

- `cargo fmt --check` 通过。
- `cargo test --all-targets` 通过；其中 `tests/paper_reproduction.rs`、CLI、portable graph、reference compare 等集成测试均通过。
- `cargo clippy --all-targets -- -D warnings` 通过。
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` 通过。
- `cargo test --test paper_reproduction` 单独通过，3 个 paper-basis witness 测试全部成功。
- `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md` 通过；本地产物为 `eml-rs.paper-reproduction-summary.v2`，catalog 覆盖率 `36/36`，`missing=0`，`partial=0`。
- `python scripts/sr_research_benchmark.py --samples 41 --steps 80 --output-json target/sr-research-benchmark.json --output-md target/sr-research-benchmark.md` 通过；本地产物为 `eml-rs.sr-research-benchmark.v2`，包含 `snapping_rules` 与 `task_metrics`。
- 使用 `gh` 下载了 run `25716323482` 的 `nightly-paper-reproduction-summary` 与 `nightly-sr-research-summary`，确认 artifact 可下载。该远端 run 来自当前远端 `main` 的旧提交，产物仍是 v1 schema；v2 schema 的多轮远端稳定性需要在本次变更合入并触发新的 nightly / `workflow_dispatch` 后继续核验。
- 变更合入 `main` 后，手动触发 `nightly.yml` 的 `paper-reproduction-only` 严格模式 run `25736267664` 与 `sr-research-only` run `25736269954`，二者均在提交 `63b59a3` 上通过；下载到的远端产物分别为 `eml-rs.paper-reproduction-summary.v2` 与 `eml-rs.sr-research-benchmark.v2`。paper summary 覆盖率为 `36/36`，`missing=0`，`partial=0`；SR summary 包含 3 个任务、45 个 runs，artifact policy 为 `nightly-and-workflow-dispatch-non-blocking`。多轮 scheduled nightly 稳定性仍需继续积累。
- `v0.2.0` annotated tag 已创建并推送；tag 解引用到提交 `6d80d31d17e66f6e04226d332107733a5df21f81`。`release.yml` run `25736978435` 通过，`validate-release-tag`、三平台 `build-release` 与 `publish-release` jobs 均成功。GitHub Release 已发布：`https://github.com/fangfuzha/eml-rs/releases/tag/v0.2.0`，状态为非 draft、非 prerelease。
- 下载并检查了 `eml-rs-v0.2.0-macos-latest.zip`、`eml-rs-v0.2.0-ubuntu-latest.zip`、`eml-rs-v0.2.0-windows-latest.zip`。三者均包含 `include/eml_rs.h` 与对应平台的 `target/release` 产物：macOS 包含 `libeml_rs.dylib`，Ubuntu 包含 `libeml_rs.so`，Windows 包含 `eml_rs.dll` 与 import/debug 产物。

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
- After downloading artifacts, run `python scripts/nightly_artifact_audit.py --artifact-root <download-dir> --require-paper-all-covered --require-sr-non-blocking --output-json target/nightly-artifact-audit.json --output-md target/nightly-artifact-audit.md` to verify v2 schemas, paper coverage, and the SR non-blocking policy.
- `README.md`, `docs/user-guide.md`, `docs/developer-guide.md`, and `docs/releases/v0.2.0/` must all point users toward the `paper-basis` entry points.
- Release notes must explicitly state that `repo-extension` training templates are outside any paper-basis completeness claim.

### 2026-05-12 Local Verification Record

- `cargo fmt --check` passed.
- `cargo test --all-targets` passed, including `tests/paper_reproduction.rs`, CLI, portable graph, and reference-compare integration tests.
- `cargo clippy --all-targets -- -D warnings` passed.
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` passed.
- `cargo test --test paper_reproduction` passed separately, with all 3 paper-basis witness tests successful.
- `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md` passed; the local artifact is `eml-rs.paper-reproduction-summary.v2` with catalog coverage `36/36`, `missing=0`, and `partial=0`.
- `python scripts/sr_research_benchmark.py --samples 41 --steps 80 --output-json target/sr-research-benchmark.json --output-md target/sr-research-benchmark.md` passed; the local artifact is `eml-rs.sr-research-benchmark.v2` and includes `snapping_rules` plus `task_metrics`.
- `gh` successfully downloaded `nightly-paper-reproduction-summary` and `nightly-sr-research-summary` from run `25716323482`. That remote run comes from the current remote `main` before this change and still emits v1 schema artifacts; multi-run remote stability for v2 schema must be checked after this change lands and a new nightly / `workflow_dispatch` run completes.
- After the change landed on `main`, `nightly.yml` was manually dispatched for strict `paper-reproduction-only` run `25736267664` and `sr-research-only` run `25736269954`; both passed on commit `63b59a3`. The downloaded remote artifacts are `eml-rs.paper-reproduction-summary.v2` and `eml-rs.sr-research-benchmark.v2`. The paper summary reports `36/36` coverage with `missing=0` and `partial=0`; the SR summary contains 3 tasks, 45 runs, and `artifact_policy = nightly-and-workflow-dispatch-non-blocking`. Multi-run scheduled-nightly stability still needs more history.
- The annotated `v0.2.0` tag was created and pushed; the tag resolves to commit `6d80d31d17e66f6e04226d332107733a5df21f81`. `release.yml` run `25736978435` passed, including `validate-release-tag`, all three platform `build-release` jobs, and `publish-release`. GitHub Release is published at `https://github.com/fangfuzha/eml-rs/releases/tag/v0.2.0` and is neither a draft nor a prerelease.
- Downloaded and inspected `eml-rs-v0.2.0-macos-latest.zip`, `eml-rs-v0.2.0-ubuntu-latest.zip`, and `eml-rs-v0.2.0-windows-latest.zip`. All three contain `include/eml_rs.h` plus platform-specific `target/release` artifacts: macOS includes `libeml_rs.dylib`, Ubuntu includes `libeml_rs.so`, and Windows includes `eml_rs.dll` plus import/debug artifacts.
