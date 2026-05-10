# Testing And Quality / 测试与质量

## 中文

### 覆盖率门槛

- 行覆盖率 `>= 80%`
- region 覆盖率 `>= 70%`
- 由 CI 中的 `cargo llvm-cov` 阻断

### 集成测试矩阵

| 场景                       | 文件                         |
| -------------------------- | ---------------------------- |
| 参考实现与训练模板         | `tests/reference_compare.rs` |
| 特殊值与分支策略           | `tests/policy_semantics.rs`  |
| 外部后端对照               | `tests/cross_backend.rs`     |
| 高层 API / 插件 / observer | `tests/api_pipeline.rs`      |

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
- `parallel-bench-only` 会额外产出 `target/parallel-bench-summary.json` 并上传 artifact，包含 Tree/RPN 阈值比值以及 Bytecode `off/auto/force` 三路对比摘要。

### 依赖安全与许可证

- `cargo audit`
- `cargo deny check licenses bans`

## English

### Coverage Thresholds

- Line coverage `>= 80%`
- Region coverage `>= 70%`
- Enforced in CI via `cargo llvm-cov`

### Integration Test Matrix

| Scenario                                  | File                         |
| ----------------------------------------- | ---------------------------- |
| Reference behavior and training templates | `tests/reference_compare.rs` |
| Special values and branch policy          | `tests/policy_semantics.rs`  |
| External backend comparison               | `tests/cross_backend.rs`     |
| High-level API / plugins / observers      | `tests/api_pipeline.rs`      |

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
- `parallel-bench-only` also emits `target/parallel-bench-summary.json` as an artifact so future threshold tuning can consume machine-readable comparisons instead of scraping logs.

### Dependency Security And Licensing

- `cargo audit`
- `cargo deny check licenses bans`
