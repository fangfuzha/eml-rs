# Acceptance Criteria / 验收标准

## 中文

本文件把 `TODO.md` 中已冻结的范围、性能目标和质量门槛转成可执行验收表。  
每条规则都会标明当前属于 `CI enforced`（自动阻断）还是 `manual audit`（人工核验）。

### 功能验收

| 类别           | 验收规则                                                                             | 执行方式                                              |
| -------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------- |
| 论文基集复现   | 代表性 paper-basis witness 必须通过纯 EML、lowering、source reference 三方一致性回放 | `CI enforced`：`cargo test --test paper_reproduction` |
| 统一表示       | 常见初等函数与已列出的 AI 激活/损失模板可降级到纯 EML IR                             | `CI enforced`：`cargo test --all-targets`             |
| 自动微分       | `symbolic_derivative` 生成的表达式经过简化后保持数值一致                             | `CI enforced`：集成测试                               |
| 表达式规模控制 | 简化后的导数表达式节点数不高于 naive 版本的 60%                                      | `CI enforced`：集成测试                               |
| 向量模板       | softmax / cross-entropy / label smoothing / focal loss 模板可构造并求值              | `CI enforced`：集成测试                               |
| 反降级         | `Expr -> SourceExpr` 的回升结果与原 EML 语义一致                                     | `CI enforced`：集成测试                               |

### 论文复现验收

| 类别           | 验收规则                                                                                      | 执行方式                                                                                                          |
| -------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 基集目录       | `paper-basis` 与 `repo-extension` 分类必须以 `docs/paper-basis-catalog.json` 为机器可读事实源 | `CI enforced`：`tests/paper_reproduction.rs` 读取 catalog 并核对 replayed witness 条目                            |
| 见证式回放     | 第一批代表性 witness 覆盖 `exp/ln/+/-/*///pow`                                                | `CI enforced`：`cargo test --test paper_reproduction`                                                             |
| 采样域覆盖     | 回归样本覆盖正实轴、负实轴、零邻域和复平面                                                    | `CI enforced`：`cargo test --test paper_reproduction`                                                             |
| 复现 artifact  | 每次 nightly 可生成 JSON 与 Markdown 摘要，用于审计但暂不阻断主流程                           | `non-blocking artifact`：`python3 scripts/paper_reproduction_summary.py` -> `target/paper-reproduction-summary.*` |
| 升级 gate 条件 | artifact 历史稳定后，再决定是否把摘要检查升级为阻断式 gate                                    | `manual governance decision`                                                                                      |

### 治理链路

| 链路             | 资产/入口                                                                                                    | 门禁策略                                     |
| ---------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------- |
| 论文复现资产     | `docs/paper-basis-catalog.*`, `tests/paper_reproduction.rs`, `target/paper-reproduction-summary.*`           | 测试阻断，摘要先非阻断 artifact              |
| 运行时性能门禁   | `benchmarks/gate.json`, `benches/`, `scripts/bench_gate.py`, `target/eml-metrics.json`                       | 关键 benchmark 阻断；指标工具按场景手动/夜间 |
| 符号回归研究结果 | `examples/symbolic_regression_loop.rs`, `scripts/sr_research_benchmark.py`, `target/sr-research-benchmark.*` | workflow_dispatch / nightly 非阻断 artifact  |

### v0.2.0 论文复现目标评估

建议把 `v0.2.0` 的研究目标定义为“论文复现可审计”：至少冻结 `paper-basis` catalog schema、保留代表性 witness replay、持续产出 nightly artifact，并把 `repo-extension` 训练模板与运行时性能门禁保持分离。

### 性能验收

| 类别                     | 门槛                              | 执行方式                                                                                                                           |
| ------------------------ | --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `verify + lowering` 延迟 | `10k nodes` 基准 `P99 <= 25ms`    | `CI enforced`：`lower_verify_10k_nodes`                                                                                            |
| `bytecode/tree` 比值     | `median <= 0.85` 且 `P99 <= 0.95` | `nightly/workflow_dispatch enforced`：`shared_eml_*` 与 `softmax_ce_*_batch1024` 基准门禁                                           |
| `RPN/tree` 比值          | `median <= 1.10`                  | `nightly/workflow_dispatch enforced`：`eml_ln_rpn_eval` 对比门禁                                                                    |
| 峰值内存                 | `1M nodes <= 1.0GB`               | `manual tool enforced`：`python3 scripts/collect_metrics.py --rss-nodes 1000000 --require-rss`；nightly 使用 `100k nodes` 轻量采样 |
| 冷启动                   | `<= 800ms`                        | `manual tool enforced`：`python3 scripts/collect_metrics.py` 输出 `cold_start.median_ms` 并按阈值返回状态                          |

### 质量验收

| 类别           | 门槛                                                                           | 执行方式                        |
| -------------- | ------------------------------------------------------------------------------ | ------------------------------- |
| 格式与静态检查 | `cargo fmt -- --check` 与 `cargo clippy --all-targets -- -D warnings` 必须通过 | `CI enforced`                   |
| 单元/集成测试  | `cargo test --all-targets` 必须通过                                            | `CI enforced`                   |
| 覆盖率         | 行覆盖率 `>= 80%`，region 覆盖率 `>= 70%`                                      | `CI enforced`：`cargo llvm-cov` |
| 跨平台兼容     | Linux / Windows / macOS 上测试必须通过                                         | `CI enforced`：workflow matrix  |
| `no_std` 分层  | `eml-core` 必须可通过 `thumbv7em-none-eabihf` 检查                             | `CI enforced`                   |

### 文档验收

| 类别         | 门槛                                                                    | 执行方式       |
| ------------ | ----------------------------------------------------------------------- | -------------- |
| 双语同步     | `README.md`、`docs/scope.md`、`docs/acceptance.md` 至少保持中英双语同步 | `manual audit` |
| 理论映射     | 论文摘要与工程边界文档必须可独立指导实现                                | `manual audit` |
| 示例可发现性 | README 中必须给出文档与示例入口                                         | `manual audit` |

### 交付验收

| 类别           | 门槛                                           | 执行方式                                  |
| -------------- | ---------------------------------------------- | ----------------------------------------- |
| Rust crate     | 根 crate 与 workspace 成员可构建、可测试       | `CI enforced`                             |
| C ABI          | `cdylib` 构建路径保持可用                      | `CI enforced`：`cargo build --release -q` |
| Benchmark gate | 缺少必需基准或性能退化超过阈值时必须阻断       | `nightly/workflow_dispatch enforced`      |
| 版本现实约束   | 本项目是个人维护项目，发布节奏是目标而非硬 SLA | `manual audit`                            |

### 说明

- 当前基准门禁基于 Criterion 的 `sample.json` 计算每次迭代归一化后的 `P95/P99`，并使用 `tukey.json` 的 mild fence 剔除测量异常值，用于近似稳定的 tail latency。
- `1k/10k/100k nodes` 与 `batch 32/256/1024/4096` 的基准覆盖已接入；其中目前自动阻断的重点仍是 `10k nodes` 与 `batch1024` 关键门槛。
- 内存与冷启动指标已接入 `scripts/collect_metrics.py`，输出 `target/eml-metrics.json`；nightly/workflow_dispatch 使用轻量规模采样，`1M nodes` 仍作为发布前人工工具门禁运行。

## English

This document turns the frozen scope, performance goals, and quality thresholds from `TODO.md` into executable acceptance criteria.  
Each rule is labeled as either `CI enforced` or `manual audit`.

### Functional Acceptance

| Category                  | Rule                                                                                                          | Enforcement                                           |
| ------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| Paper-basis reproduction  | Representative paper-basis witnesses must replay consistently across pure EML, lowering, and source reference | `CI enforced`: `cargo test --test paper_reproduction` |
| Unified representation    | Common elementary functions and the listed AI activation/loss templates must lower into pure EML IR           | `CI enforced`: `cargo test --all-targets`             |
| Autodiff                  | `symbolic_derivative` outputs remain numerically consistent after simplification                              | `CI enforced`: integration tests                      |
| Expression growth control | Simplified derivative trees must stay within 60% of naive node count                                          | `CI enforced`: integration tests                      |
| Vector templates          | softmax / cross-entropy / label smoothing / focal loss templates must build and evaluate successfully         | `CI enforced`: integration tests                      |
| De-lowering               | `Expr -> SourceExpr` must preserve the original EML semantics                                                 | `CI enforced`: integration tests                      |

### Paper Reproduction Acceptance

| Category              | Rule                                                                                                                               | Enforcement                                                                                                       |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Basis catalog         | `paper-basis` and `repo-extension` classification must use `docs/paper-basis-catalog.json` as the machine-readable source of truth | `CI enforced`: `tests/paper_reproduction.rs` reads the catalog and checks replayed witness entries                |
| Witness replay        | The first representative witness batch covers `exp/ln/+/-/*///pow`                                                                 | `CI enforced`: `cargo test --test paper_reproduction`                                                             |
| Domain coverage       | Regression samples cover the positive real axis, negative real axis, zero neighborhood, and complex plane                          | `CI enforced`: `cargo test --test paper_reproduction`                                                             |
| Reproduction artifact | Nightly can generate JSON and Markdown summaries for audit without blocking the main pipeline                                      | `non-blocking artifact`: `python3 scripts/paper_reproduction_summary.py` -> `target/paper-reproduction-summary.*` |
| Gate promotion        | Promote summary checks to a blocking gate only after artifact history is stable                                                    | `manual governance decision`                                                                                      |

### Governance Tracks

| Track                        | Assets / Entry Points                                                                                        | Gate Policy                                        |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| Paper reproduction assets    | `docs/paper-basis-catalog.*`, `tests/paper_reproduction.rs`, `target/paper-reproduction-summary.*`           | blocking tests, non-blocking summary artifacts     |
| Runtime performance gates    | `benchmarks/gate.json`, `benches/`, `scripts/bench_gate.py`, `target/eml-metrics.json`                       | key benchmarks block; metrics run manually/nightly |
| Symbolic-regression research | `examples/symbolic_regression_loop.rs`, `scripts/sr_research_benchmark.py`, `target/sr-research-benchmark.*` | workflow_dispatch / nightly non-blocking artifacts |

### v0.2.0 Paper-Reproducibility Target

Recommended `v0.2.0` research target: auditable paper reproduction. At minimum, freeze the `paper-basis` catalog schema, keep representative witness replay, emit nightly artifacts, and keep `repo-extension` training templates separate from runtime performance gates.

### Performance Acceptance

| Category                    | Threshold                                   | Enforcement                                                                                                                                    |
| --------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `verify + lowering` latency | `P99 <= 25ms` for the `10k nodes` benchmark | `CI enforced`: `lower_verify_10k_nodes`                                                                                                        |
| `bytecode/tree` ratio       | `median <= 0.85` and `P99 <= 0.95`          | `nightly/workflow_dispatch enforced`: `shared_eml_*` and `softmax_ce_*_batch1024` benchmark gate                                                |
| `RPN/tree` ratio            | `median <= 1.10`                            | `nightly/workflow_dispatch enforced`: `eml_ln_rpn_eval` ratio gate                                                                              |
| Peak memory                 | `1M nodes <= 1.0GB`                         | `manual tool enforced`: `python3 scripts/collect_metrics.py --rss-nodes 1000000 --require-rss`; nightly uses a lightweight `100k nodes` sample |
| Cold start                  | `<= 800ms`                                  | `manual tool enforced`: `python3 scripts/collect_metrics.py` emits `cold_start.median_ms` and exits according to the threshold                 |

### Quality Acceptance

| Category                     | Threshold                                                                        | Enforcement                     |
| ---------------------------- | -------------------------------------------------------------------------------- | ------------------------------- |
| Formatting and lint          | `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` must pass | `CI enforced`                   |
| Unit/integration tests       | `cargo test --all-targets` must pass                                             | `CI enforced`                   |
| Coverage                     | Line coverage `>= 80%`, region coverage `>= 70%`                                 | `CI enforced`: `cargo llvm-cov` |
| Cross-platform compatibility | Tests must pass on Linux / Windows / macOS                                       | `CI enforced`: workflow matrix  |
| `no_std` layering            | `eml-core` must build for `thumbv7em-none-eabihf`                                | `CI enforced`                   |

### Documentation Acceptance

| Category                      | Threshold                                                                                            | Enforcement    |
| ----------------------------- | ---------------------------------------------------------------------------------------------------- | -------------- |
| Bilingual sync                | `README.md`, `docs/scope.md`, and `docs/acceptance.md` must stay synchronized in Chinese and English | `manual audit` |
| Theory-to-engineering mapping | The paper summary and boundary docs must be sufficient to guide implementation                       | `manual audit` |
| Discoverability               | README must link to the documentation and examples                                                   | `manual audit` |

### Delivery Acceptance

| Category                     | Threshold                                                                         | Enforcement                               |
| ---------------------------- | --------------------------------------------------------------------------------- | ----------------------------------------- |
| Rust crate                   | The root crate and workspace members must build and test successfully             | `CI enforced`                             |
| C ABI                        | The `cdylib` build path must remain valid                                         | `CI enforced`: `cargo build --release -q` |
| Benchmark gate               | Missing required benchmarks or threshold regressions must block the pipeline      | `nightly/workflow_dispatch enforced`      |
| Personal-maintenance reality | This is a personal project, so release cadence is a target rather than a hard SLA | `manual audit`                            |

### Notes

- The benchmark gate derives normalized `P95/P99` from Criterion `sample.json` and uses the `tukey.json` mild fences to drop measurement outliers, yielding a more stable tail-latency approximation for CI.
- Coverage for `1k/10k/100k nodes` and `batch 32/256/1024/4096` is now wired into the benchmark suite; the current blocking gates still focus on the `10k nodes` and `batch1024` thresholds first.
- Memory and cold-start metrics are wired through `scripts/collect_metrics.py`, which writes `target/eml-metrics.json`; nightly/workflow_dispatch uses a lightweight sample, while `1M nodes` remains a pre-release manual-tool gate.
