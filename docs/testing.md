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
- 第一批 `VerifyBaseSet` 风格轻量 harness 回放 `exp`、`ln`、`+`、`-`、`*`、`/`、`pow` 的代表性 witness 链。
- 每条 witness 都比较三方输出：纯 EML witness、source lowering 结果、source reference。
- 样本域固定覆盖正实轴、负实轴、零邻域与复平面。
- `scripts/paper_reproduction_summary.py` 产出 `target/paper-reproduction-summary.json` 与 `target/paper-reproduction-summary.md`；nightly 上传 artifact，先不把摘要升级为阻断式 gate。

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

- 每次 PR 运行 `cargo bench --bench eval_bench`
- 用 `scripts/bench_gate.py` 对比门槛
- 基准覆盖包含 `softmax_ce_*_batch32/256/1024/4096` 与 `lower_verify_1k/10k/100k_nodes`
- 重点门禁仍聚焦：`shared_eml_*`、`softmax_ce_*_batch1024`、`lower_verify_10k_nodes`
- Tree/RPN 阈值拐点与 Bytecode 并行候选评估已拆到 `cargo bench --bench parallel_bench`；Linux 上使用 `nightly.yml` 的 `parallel-bench-only` 手动入口运行，不直接纳入主 gate。
- `parallel-bench-only` 会额外产出 `target/parallel-bench-summary.json` 并上传 artifact，包含 Tree/RPN 阈值比值、Bytecode `off/auto/force` 三路对比摘要，以及当前默认策略配置与各 batch 的 median winner。
- workflow 日志还会打印 `[parallel-summary] ...` 的简明推荐行，便于快速判断是否需要复审默认并行策略；如后续要门禁化，可直接复用脚本的 `--require-*` 选项。

### 符号回归研究基准

- `scripts/sr_research_benchmark.py` 是 P20 的独立研究面，固定深度 `2..6`、样本规模、初始化策略与 hardening 参数。
- 输出指标包括 `recovery_rate`、`snap_to_symbolic_rate`、`nan_overflow_incidence` 与 `wall_time_ms`。
- 默认产物是 `target/sr-research-benchmark.json` 与 `target/sr-research-benchmark.md`。
- Linux nightly / `workflow_dispatch` 上传该产物作为非阻断 artifact，不纳入主 CI 必过门禁。

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
- The first lightweight `VerifyBaseSet`-style harness replays representative witness chains for `exp`, `ln`, `+`, `-`, `*`, `/`, and `pow`.
- Each witness compares three outputs: pure EML witness, source lowering result, and source reference.
- The fixed sample set covers the positive real axis, negative real axis, zero neighborhood, and complex plane.
- `scripts/paper_reproduction_summary.py` emits `target/paper-reproduction-summary.json` and `target/paper-reproduction-summary.md`; nightly uploads them as artifacts before any decision to promote summaries into a blocking gate.

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

- Run `cargo bench --bench eval_bench` on every PR
- Enforce thresholds with `scripts/bench_gate.py`
- Coverage includes `softmax_ce_*_batch32/256/1024/4096` and `lower_verify_1k/10k/100k_nodes`
- Primary blocking workloads remain: `shared_eml_*`, `softmax_ce_*_batch1024`, `lower_verify_10k_nodes`
- Tree/RPN threshold probing and Bytecode parallel candidate evaluation live in `cargo bench --bench parallel_bench`; on Linux they run through the manual `parallel-bench-only` target in `nightly.yml` and stay outside the main blocking gate for now.
- `parallel-bench-only` also emits `target/parallel-bench-summary.json` as an artifact so future threshold tuning can consume machine-readable comparisons instead of scraping logs; the summary includes Tree/RPN ratios, Bytecode `off/auto/force` comparisons, the configured default policy, and per-batch median winners.
- The workflow logs also print concise `[parallel-summary] ...` recommendation lines. If this needs to become a gate later, the same script already exposes `--require-*` checks for reuse.

### Symbolic Regression Research Benchmark

- `scripts/sr_research_benchmark.py` is the independent P20 research surface with fixed depths `2..6`, sample size, initialization policy, and hardening parameters.
- Reported metrics include `recovery_rate`, `snap_to_symbolic_rate`, `nan_overflow_incidence`, and `wall_time_ms`.
- Default artifacts are `target/sr-research-benchmark.json` and `target/sr-research-benchmark.md`.
- Linux nightly / `workflow_dispatch` uploads these artifacts as non-blocking research outputs and does not include them in the required main CI gate.

### Dependency Security And Licensing

- `cargo audit`
- `cargo deny check licenses bans`
