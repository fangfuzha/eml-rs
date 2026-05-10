# Observability / 可观测性

## 中文

### 当前能力

- `PipelineReport`: 暴露源表达式节点数、IR 统计、字节码指令数。
- `PipelineObserver`: 在 `Parsed / OptimizedSource / SourcePass / Lowered / ExprPass / BytecodeCompiled` 阶段发事件。
- `EvalMetrics` / `VerifyMetrics`: 暴露 batch 耗时、`parallel` 标记和 `workers`，可用于判断默认策略是否触发了样本级并行。
- benchmark gate: 记录 `median/P95/P99` 与比值门槛。

### 当前不做的事

- 不内置日志框架依赖。
- 不内置 Prometheus/OpenTelemetry SDK。
- 不做服务端 tracing 方案，因为当前项目不是服务框架。

### 推荐做法

- 库内实验：直接挂 `PipelineObserver` 收集阶段事件。
- CI/性能治理：依赖 `criterion + bench_gate.py`。
- 交互式对照：通过 CLI 的 `--bytecode-parallel off|auto|force` 配合 `eval_parallel` / `eval_workers` 输出观察默认策略切换点。当前 Bytecode `auto` 的默认值为 `workers <= 8`、`min_samples_per_worker = 128`。
- 未来如要对接服务框架，再由宿主系统把 `PipelineReport` 映射到日志、指标或 trace。

## English

### Current Capability

- `PipelineReport` exposes source-node counts, IR stats, and bytecode instruction counts.
- `PipelineObserver` emits events for `Parsed / OptimizedSource / SourcePass / Lowered / ExprPass / BytecodeCompiled`.
- `EvalMetrics` / `VerifyMetrics` expose batch timings plus `parallel` and `workers`, which makes it possible to see when the default strategy switched into sample-level parallel execution.
- The benchmark gate tracks `median/P95/P99` and ratio thresholds.

### What Is Not Built In

- No bundled logging framework dependency.
- No bundled Prometheus/OpenTelemetry SDK.
- No service-side tracing stack, because this project is not a serving framework.

### Recommended Practice

- In-library experiments: attach a `PipelineObserver`.
- CI and performance governance: rely on `criterion + bench_gate.py`.
- Interactive comparison: use the CLI `--bytecode-parallel off|auto|force` switch together with `eval_parallel` / `eval_workers` to inspect when the default strategy changes mode. The current Bytecode `auto` defaults are `workers <= 8` and `min_samples_per_worker = 128`.
- If this is embedded into a service later, let the host system map `PipelineReport` into logs, metrics, or traces.
