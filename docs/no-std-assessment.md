# `no_std` 可行性评估（eml-rs）

## 结论
- 当前代码形态**不直接支持** `no_std`。
- 通过分层拆分后可实现“核心算子支持 `no_std`，高层功能保留 `std`”的路线。

## 主要阻塞点
1. 当前错误与展示依赖 `std::error::Error` / `Display`。
2. 解析器与编译模块大量使用 `String`、`Vec`、`HashMap`（需要 `alloc`，部分需要 `std`）。
3. FFI 与进程级测试（Python 后端对照）天然是 `std` 场景。

## 可行拆分方案
1. `eml-core`（目标 `no_std + alloc`）
   - 保留 `core` 的数值算子与策略结构体。
   - 使用 `libm`/`num-traits` 等兼容层（视目标平台而定）。
2. `eml-engine`（`std`）
   - 保留 `lowering` / `opt` / `verify` / `bytecode` 高级功能。
3. `eml-ffi`（`std`）
   - 保留 `cdylib` 导出与跨语言集成。

## 迁移步骤建议
1. 先将 `core` 独立成单独 crate，并去除对 `std` 的强绑定。
2. 将解析、验证、跨后端测试全部隔离到 `std` crate。
3. 为嵌入式目标增加最小回归集：`eml_real`, `eml_complex`, `policy` 语义一致性测试。
