# Testing And Quality / 测试与质量

## 中文

### 覆盖率门槛

- 行覆盖率 `>= 80%`
- region 覆盖率 `>= 70%`
- 由 CI 中的 `cargo llvm-cov` 阻断

### 集成测试矩阵

| 场景                        | 文件                          |
| --------------------------- | ----------------------------- |
| 参考实现与训练模板          | `tests/reference_compare.rs`  |
| 论文基集复现与 witness 回放 | `tests/paper_reproduction.rs` |
| 特殊值与分支策略            | `tests/policy_semantics.rs`   |
| 外部后端对照                | `tests/cross_backend.rs`      |
| 高层 API / 插件 / observer  | `tests/api_pipeline.rs`       |

### 论文复现测试

- `tests/paper_reproduction.rs` 是 `paper-basis` 专用测试面，不混入 `repo-extension` 训练模板。
- `VerifyBaseSet` 风格轻量 harness 回放 catalog 标注的代表性 witness 链，当前覆盖 `exp`、`ln`、`+`、`-`、`*`、`/`、`pow`、P22 新增缺口与组合命名入口。
- 每条 witness 都比较三方输出：纯 EML witness、source lowering 结果、source reference。
- 样本域固定覆盖正实轴、负实轴、零邻域与复平面。
- `scripts/paper_reproduction_summary.py` 从 `docs/paper-basis-catalog.json` 消费 replayed witness 锚点，产出 schema v2 的 `target/paper-reproduction-summary.json` 与 `target/paper-reproduction-summary.md`，包含覆盖率、missing/partial 明细与 witness provenance。
- nightly 默认继续上传非阻断 artifact；`workflow_dispatch` 可用 `paper_strict_gate=true` 手动启用 `--require-all-covered --require-no-missing-replayed --require-min-covered-ratio 1.0`。

### 兼容性矩阵

- OS: Linux / Windows / macOS
- 架构检查: `x86_64` + `aarch64` target check
- Rust 版本: stable + MSRV `1.86`
- `no_std`: `eml-core` on `thumbv7em-none-eabihf`

### Fuzzing

- `fuzz/parse_lower_eval.rs`: parser -> lowering -> eval 路线
- `fuzz/expr_eval_consistency.rs`: tree / RPN / bytecode 一致性
- `fuzz/autodiff_simplify.rs`: 求导与简化路径

### 性能回归

- 主 CI 跑格式、clippy、测试、覆盖率、兼容性与供应链检查；性能门禁由 nightly / `workflow_dispatch` 的 `bench-only` 路径运行。
- P27 决策：暂不新增 PR 级 smoke benchmark。当前 Criterion 子集仍会增加 PR 时长且容易受共享 runner 抖动影响，运行时性能回退继续由 nightly 与手动 `bench-only` 阻断；待多轮 artifact 历史稳定后再复审是否提升到 PR 级信号。
- `bench-only` 运行 `cargo bench --bench eval_bench`，并用 `scripts/bench_gate.py` 对比门槛。
- 基准覆盖包含 `softmax_ce_*_batch32/256/1024/4096` 与 `lower_verify_1k/10k/100k_nodes`
- 重点门禁仍聚焦：`shared_eml_*`、`softmax_ce_*_batch1024`、`lower_verify_10k_nodes`
- Tree/RPN 阈值拐点与 Bytecode 并行候选评估已拆到 `cargo bench --bench parallel_bench`；Linux 上使用 `nightly.yml` 的 `parallel-bench-only` 手动入口运行，不直接纳入主 gate。
- `parallel-bench-only` 会额外产出 `target/parallel-bench-summary.json` 并上传 artifact，包含 Tree/RPN 阈值比值、Bytecode `off/auto/force` 三路对比摘要，以及当前默认策略配置与各 batch 的 median winner。
- workflow 日志还会打印 `[parallel-summary] ...` 的简明推荐行，便于快速判断是否需要复审默认并行策略；如后续要门禁化，可直接复用脚本的 `--require-*` 选项。

### 符号回归研究基准

- `scripts/sr_research_benchmark.py` 是 P20/P24 的独立研究面，当前覆盖 `exp-log`、三角模板、低阶多项式模板三个固定任务族。
- 默认深度桶仍为 `2..6`，但每个任务会对 seed 集合重复试验，并按任务和全局两个层级聚合结果。
- schema 已升级为 `eml-rs.sr-research-benchmark.v2`；JSON 同时包含 `tasks`、`task_metrics`、逐 run `runs`、`failure_summary` 与 `snapping_rules`，避免脚本输出成为隐式协议。
- 聚合指标包括 `recovery_rate`、`snap_to_symbolic_rate`、`final_loss_mean/variance`、`param_rmse_mean/variance`、`nan_overflow_incidence_mean/variance` 与 `wall_time_ms_mean/variance`，并给出 best/worst run 与失败样本摘要。
- snapping 规则显式区分：参数近似阈值 `param_rmse <= 0.20`、数值等价采样域 `x in [-2, 2]`、最大绝对误差阈值 `1e-3`、以及“数值等价但参数未对齐”的 `numerically-equivalent-indeterminate` 状态。
- 默认产物仍是 `target/sr-research-benchmark.json` 与 `target/sr-research-benchmark.md`。
- Linux nightly / `workflow_dispatch` 继续上传该产物作为非阻断 artifact，不纳入主 CI 必过门禁，也不与 runtime 性能 gate 混合。

### 论文发现搜索 provenance

- P30 将最短式 / 搜索 provenance 先定义为治理协议，不在当前阶段实现搜索 harness。
- 未来搜索 artifact 必须声明 `proof_level`：只有 `exhaustive-bounded` 能表达“给定边界内最短”，`heuristic` 与 `sampled` 只能表达“当前搜索找到的最优候选”。
- 未来搜索 artifact 必须记录 `search_space`、`objective`、`candidate`、`validation` 与 `non_goals`，并引用 `docs/paper-basis-catalog.json` 的 schema 和 git SHA。
- 搜索结果初期继续作为 nightly / `workflow_dispatch` 非阻断研究 artifact，不能替代 `tests/paper_reproduction.rs` 的 witness replay，也不能影响 runtime 性能 gate。

### 依赖安全与许可证

- `cargo audit`
- `cargo deny check licenses bans`

## English

### Coverage Thresholds

- Line coverage `>= 80%`
- Region coverage `>= 70%`
- Enforced in CI via `cargo llvm-cov`

### Integration Test Matrix

| Scenario                                    | File                          |
| ------------------------------------------- | ----------------------------- |
| Reference behavior and training templates   | `tests/reference_compare.rs`  |
| Paper-basis reproduction and witness replay | `tests/paper_reproduction.rs` |
| Special values and branch policy            | `tests/policy_semantics.rs`   |
| External backend comparison                 | `tests/cross_backend.rs`      |
| High-level API / plugins / observers        | `tests/api_pipeline.rs`       |

### Paper Reproduction Tests

- `tests/paper_reproduction.rs` is the dedicated `paper-basis` test surface and does not mix in `repo-extension` training templates.
- The lightweight `VerifyBaseSet`-style harness replays catalog-marked representative witness chains, currently covering `exp`, `ln`, `+`, `-`, `*`, `/`, `pow`, the P22 gap fills, and composition-only paper-basis names.
- Each witness compares three outputs: pure EML witness, source lowering result, and source reference.
- The fixed sample set covers the positive real axis, negative real axis, zero neighborhood, and complex plane.
- `scripts/paper_reproduction_summary.py` consumes replayed witness anchors from `docs/paper-basis-catalog.json` and emits schema v2 `target/paper-reproduction-summary.json` plus `target/paper-reproduction-summary.md` with coverage ratio, missing/partial details, and witness provenance.
- Nightly still uploads a non-blocking artifact by default; `workflow_dispatch` can set `paper_strict_gate=true` to enable `--require-all-covered --require-no-missing-replayed --require-min-covered-ratio 1.0` manually.

### Compatibility Matrix

- OS: Linux / Windows / macOS
- Architecture checks: `x86_64` + `aarch64` target checks
- Rust versions: stable + MSRV `1.86`
- `no_std`: `eml-core` on `thumbv7em-none-eabihf`

### Fuzzing

- `fuzz/parse_lower_eval.rs`: parser -> lowering -> eval path
- `fuzz/expr_eval_consistency.rs`: tree / RPN / bytecode consistency
- `fuzz/autodiff_simplify.rs`: autodiff and simplification path

### Performance Regression

- Main CI runs formatting, clippy, tests, coverage, compatibility, and supply-chain checks; performance gates run through nightly / `workflow_dispatch` `bench-only`.
- P27 decision: do not add a PR-level smoke benchmark yet. A Criterion subset would still add PR latency and remain noisy on shared runners, so runtime regressions continue to block through nightly and manual `bench-only`; revisit PR-level signaling after more artifact history is stable.
- `bench-only` runs `cargo bench --bench eval_bench` and enforces thresholds with `scripts/bench_gate.py`.
- Coverage includes `softmax_ce_*_batch32/256/1024/4096` and `lower_verify_1k/10k/100k_nodes`
- Primary blocking workloads remain: `shared_eml_*`, `softmax_ce_*_batch1024`, `lower_verify_10k_nodes`
- Tree/RPN threshold probing and Bytecode parallel candidate evaluation live in `cargo bench --bench parallel_bench`; on Linux they run through the manual `parallel-bench-only` target in `nightly.yml` and stay outside the main blocking gate for now.
- `parallel-bench-only` also emits `target/parallel-bench-summary.json` as an artifact so future threshold tuning can consume machine-readable comparisons instead of scraping logs; the summary includes Tree/RPN ratios, Bytecode `off/auto/force` comparisons, the configured default policy, and per-batch median winners.
- The workflow logs also print concise `[parallel-summary] ...` recommendation lines. If this needs to become a gate later, the same script already exposes `--require-*` checks for reuse.

### Symbolic Regression Research Benchmark

- `scripts/sr_research_benchmark.py` is the independent P20/P24 research surface and now covers three fixed task families: `exp-log`, trigonometric, and low-order polynomial templates.
- The default depth buckets remain `2..6`, but each task now repeats across a seed set and reports both per-task and cross-task aggregates.
- The schema is now `eml-rs.sr-research-benchmark.v2`; the JSON artifact explicitly records `tasks`, `task_metrics`, per-run `runs`, `failure_summary`, and `snapping_rules` so the output is a documented contract instead of an implicit script byproduct.
- Aggregate metrics include `recovery_rate`, `snap_to_symbolic_rate`, `final_loss_mean/variance`, `param_rmse_mean/variance`, `nan_overflow_incidence_mean/variance`, and `wall_time_ms_mean/variance`, plus best/worst runs and failed-run examples.
- The snapping policy now distinguishes parameter closeness `param_rmse <= 0.20`, numerical equivalence over `x in [-2, 2]`, a max absolute error threshold of `1e-3`, and the `numerically-equivalent-indeterminate` state for numerically matched but parameter-divergent runs.
- Default artifacts remain `target/sr-research-benchmark.json` and `target/sr-research-benchmark.md`.
- Linux nightly / `workflow_dispatch` continues to upload these artifacts as non-blocking research outputs and keeps them separate from the required runtime-performance gate.

### Paper-Discovery Search Provenance

- P30 defines shortest-expression / search provenance as a governance protocol first; the current phase does not implement a search harness.
- Future search artifacts must declare `proof_level`: only `exhaustive-bounded` may say “shortest within the declared bounds,” while `heuristic` and `sampled` may only say “best candidate found by the current search.”
- Future search artifacts must record `search_space`, `objective`, `candidate`, `validation`, and `non_goals`, and cite the `docs/paper-basis-catalog.json` schema plus git SHA.
- Search results start as nightly / `workflow_dispatch` non-blocking research artifacts. They must not replace the `tests/paper_reproduction.rs` witness replay and must not affect runtime performance gates.

### Dependency Security And Licensing

- `cargo audit`
- `cargo deny check licenses bans`
