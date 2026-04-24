# v0.1.1 Compatibility Summary / 兼容摘要

## 中文

### 版本与工具链
- crate 版本: `0.1.1`
- MSRV: `Rust 1.86`
- edition: `2021`

### 支持矩阵
- CI 操作系统: `Linux / Windows / macOS`
- 宿主完整检查: `x86_64-unknown-linux-gnu` 上的 `--all-targets`
- 生产交叉目标检查: `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
- `no_std` 检查: `thumbv7em-none-eabihf` 上的 `eml-core`

### API 兼容
- 高层 pipeline、lowering、verify、C ABI 的核心入口保持可用。
- 新增 parallel/profile API，不移除既有串行 API。
- `0.x` 阶段仍允许 API 演进，但需要在迁移说明中明确影响面。

### 交付形态
- Rust crate
- C ABI (`cdylib`)
- Rust examples
- 最小 C 调用示例
- Criterion benchmark 与远端 benchmark gate

## English

### Version And Toolchain
- crate version: `0.1.1`
- MSRV: `Rust 1.86`
- edition: `2021`

### Supported Matrix
- CI operating systems: `Linux / Windows / macOS`
- Full host check: `--all-targets` on `x86_64-unknown-linux-gnu`
- Production cross-target checks: `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
- `no_std` check: `eml-core` on `thumbv7em-none-eabihf`

### API Compatibility
- The core high-level pipeline, lowering, verification, and C ABI entry points remain available.
- Parallel/profile APIs are additive and do not remove existing serial APIs.
- The `0.x` line may still evolve, but migration notes must describe the impact.

### Delivery Forms
- Rust crate
- C ABI (`cdylib`)
- Rust examples
- Minimal C invocation example
- Criterion benchmarks and remote benchmark gates
