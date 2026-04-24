# v0.1.1 Release Notes / 发布说明

## 中文

`v0.1.1` 是 `eml-rs` 的第一个补丁发布准备快照，重点是把 `v0.1.0` 之后的性能剖析、样本级并行、bytecode batch 成本优化，以及 `criterion 0.8` 依赖升级收口到可发布状态。

### 本版重点
- 新增 pipeline profiling 能力，将 lowering、simplify、bytecode build、eval、verify 分段计时。
- 新增样本级并行验证 API：`verify_against_*_ref_parallel` 与 `profile_verify_against_*_ref_parallel`。
- 新增 Tree/RPN 的样本级并行 batch eval API：`eval_complex_batch_parallel`、`eval_real_batch_parallel` 及对应 profile 方法。
- 优化 bytecode batch 路径，复用临时 buffer，降低单样本执行成本。
- 修正 `lower_verify_10k_nodes` 的样本域与容差，使其同时具备吞吐和正确性可读性。
- 升级 dev benchmark 依赖 `criterion` 到 `0.8.2`，并将 benchmark 中的 `black_box` 切到 `std::hint::black_box`。
- 修复远端 compat CI：宿主目标继续检查 `--all-targets`，非宿主交叉目标只检查生产构建面，避免 dev-dependency 的 C 编译链污染交叉编译。

### 当前边界
- Bytecode batch 已优先做单样本成本优化；并行化仍需基于 profiling 数据再决定。
- GPU/NPU 后端仍不是本版本目标；本版本只完善 CPU 单机实验、验证与 bench 基线。
- `criterion 0.8.2` 只用于 dev/bench，不应进入运行时依赖面。

## English

`v0.1.1` is the first patch release-preparation snapshot after `v0.1.0`. It consolidates profiling, sample-level parallelism, bytecode batch-cost improvements, and the `criterion 0.8` dependency upgrade into a publishable baseline.

### Highlights
- Added pipeline profiling for lowering, simplification, bytecode build, evaluation, and verification stages.
- Added sample-level parallel verification APIs: `verify_against_*_ref_parallel` and `profile_verify_against_*_ref_parallel`.
- Added Tree/RPN sample-level parallel batch evaluation APIs: `eval_complex_batch_parallel`, `eval_real_batch_parallel`, and matching profile methods.
- Optimized bytecode batch evaluation by reusing temporary buffers and lowering per-sample overhead.
- Fixed the `lower_verify_10k_nodes` benchmark sample domain and tolerance so it reports both throughput and readable correctness.
- Upgraded the dev benchmark dependency `criterion` to `0.8.2`, and switched benchmark `black_box` usage to `std::hint::black_box`.
- Fixed remote compat CI: host targets still check `--all-targets`, while non-host cross targets check only the production build surface to avoid leaking dev-dependency C toolchains into cross builds.

### Current Boundary
- Bytecode batch optimization focuses on single-sample cost first; parallelization should still be driven by profiling data.
- GPU/NPU backends are still out of scope for this release; this snapshot focuses on single-machine CPU experiments, verification, and benchmark baselines.
- `criterion 0.8.2` is dev/bench-only and should not enter the runtime dependency surface.
