# v0.1.0 Compatibility Summary / 兼容摘要

## 中文

### 版本与工具链
- crate 版本: `0.1.0`
- MSRV: `Rust 1.75`
- edition: `2021`

### 支持矩阵
- CI 操作系统: `Linux / Windows / macOS`
- 目标检查: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
- `no_std` 检查: `thumbv7em-none-eabihf` 上的 `eml-core`

### 交付形态
- Rust crate
- C ABI (`cdylib`)
- 示例程序: Rust example + 最小 C 调用示例

### 稳定性说明
- `0.x` 阶段 API 允许演进，但需附迁移说明。
- 当前 release snapshot 用于冻结首个可复现基线。

## English

### Version And Toolchain
- crate version: `0.1.0`
- MSRV: `Rust 1.75`
- edition: `2021`

### Supported Matrix
- CI operating systems: `Linux / Windows / macOS`
- target checks: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
- `no_std` check: `eml-core` on `thumbv7em-none-eabihf`

### Delivery Forms
- Rust crate
- C ABI (`cdylib`)
- Examples: Rust example + minimal C invocation example

### Stability Note
- The `0.x` line may still evolve, but migration notes are required.
- This release snapshot is meant to freeze the first reproducible baseline.
