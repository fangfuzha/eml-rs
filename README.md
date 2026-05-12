# eml-rs

基于单一二元算子 `eml(x, y) = exp(x) - ln(y)` 的工程化实验仓库。  
本仓库关注的是：**统一表达、统一编译、统一验证**，而不是盲目替换所有运行时 kernel。

## 1. 这个项目在做什么

`eml-rs` 把常见数学表达式先降级到纯 EML 树，再在这个统一 IR 上做：

- 表达式搜索与重写
- 常量折叠 / CSE
- 多执行后端（树、RPN、字节码）
- 数值一致性验证（含跨后端）
- C ABI 暴露（方便嵌入其他系统）

项目目标不是“EML 一定比原生算子快”，而是先获得统一可控的中间表示，再按目标平台做最优部署。

## 文档入口 / Documentation

- 文档索引 / Docs index: `docs/README.md`
- 论文摘要与工程指导: `docs/eml-paper-summary.md`
- 论文基集目录 / Paper-basis catalog: `docs/paper-basis-catalog.md`
- 论文复现 release 草案 / Paper-reproduction release draft: `docs/releases/v0.2.0/README.md`
- 范围声明 / Scope: `docs/scope.md`
- 验收标准 / Acceptance: `docs/acceptance.md`
- 架构设计 / Architecture: `docs/architecture.md`
- 使用者指南 / User guide: `docs/user-guide.md`
- CLI 使用说明 / CLI guide: `docs/cli.md`
- API 稳定性 / API stability: `docs/api-stability.md`
- 互操作 / Interoperability: `docs/interoperability.md`
- 开发者指南 / Developer guide: `docs/developer-guide.md`
- 版本策略 / Versioning: `docs/versioning.md`
- 测试与质量 / Testing: `docs/testing.md`
- 可观测性 / Observability: `docs/observability.md`
- 维护策略 / Maintenance: `docs/maintenance.md`
- 合规说明 / Compliance: `docs/compliance.md`

## 2. EML 在各个领域的作用（你问的核心）

### 2.1 数学与符号层（表达统一）

- 作用：把大量函数写成同构树 `One | Var | Eml(lhs, rhs)`。
- 收益：表达式等价变换、搜索、枚举、结构比较更容易。
- 代价：表达深度和节点数可能膨胀，直接执行成本会变高。

### 2.2 编译器层（IR 与优化）

- 作用：把异构算子前端统一到单语义 IR，再做优化。
- 收益：优化器只需要懂一套语义，可复用 CSE、常量折叠、重写规则。
- 代价：若缺少代价模型，容易出现“语义正确但性能变差”。

### 2.3 验证与可信计算层

- 作用：统一语义后，数值对照与一致性测试可标准化。
- 收益：更容易比较 tree/RPN/bytecode/外部后端结果。
- 代价：复对数分支、NaN/Inf 语义必须显式制定策略，否则跨平台漂移明显。

### 2.4 AI 训练与推理层

- 作用：作为“统一表达层”，特别适合神经符号、可解释模型、符号回归、自动公式发现。
- 收益：训练中可统一表示激活/损失模板，便于自动变换与搜索。
- 代价：不应把深度学习高性能内核全部替换成 EML 执行。实际部署通常要反降级回原生 kernel。

### 2.5 硬件设计层

- 作用：提供“单算子视角”的硬件映射目标（可围绕 EML 设计流水线/调度）。
- 收益：控制面与编译面简化，便于做专用加速实验。
- 边界：**只优化 EML 单元不等于训练必然加速**。总体性能还受：
  - `exp/log` 本身代价
  - 图展开规模
  - 内存带宽与并行调度
  - 反向传播实现细节
  - 数值稳定策略

结论：EML 是“统一计算语言 + 编译/硬件协同抓手”，不是银弹。

## 3. 项目结构

- `src/core.rs`: EML 原子算子、分支策略、特殊值策略
- `src/error.rs`: 统一错误码、诊断信息
- `src/ir.rs`: `Expr` IR、RPN、统计信息
- `src/bytecode.rs`: 寄存器字节码执行器（含 CSE/常量折叠）
- `src/api.rs`: 高层 pipeline API（parse/opt/lower/compile/eval/verify）
- `src/plugin.rs`: 自定义 pass / backend / observer 扩展点
- `src/opt.rs`: 源表达式重写与代价模型
- `src/verify.rs`: 数值对照验证
- `src/ffi.rs`: C ABI 导出
- `src/lowering.rs`: 对独立 lowering crate 的兼容封装
- `crates/eml-lowering`: `no_std + alloc` 的 parser/lowering 独立 crate
- `tests/`: 集成测试与跨后端验证
- `examples/`: 训练环路、C FFI 最小示例

## 4. 当前支持函数族

当前函数能力分为两条治理链路：`paper-basis` 表示论文 Table 1 scientific-calculator basis 内的能力，`repo-extension` 表示仓库为了训练模板、互操作或研究工程自行扩展的能力。完整边界以 `docs/paper-basis-catalog.md` 为准。

### 4.1 Paper-basis 初等函数（已实现 lowering）

- `+ - * / pow exp log`
- `asinh acosh atanh hypot`
- `sin cos tan`
- `sinh cosh tanh`
- `asin acos atan`
- `sqrt`
- `sigmoid`（论文基集成员，同时也可服务训练模板）

说明：`half`、`inv`、`sqr`、`avg`、`log_x` 现通过公开命名入口解析为组合表达，并继续按 `paper-basis` 治理，而不是扩成新的核心运行时语义节点。完整 witness、测试锚点与覆盖状态以 `docs/paper-basis-catalog.md` / `docs/paper-basis-catalog.json` 为准。

### 4.2 Repo-extension AI 常见函数（模板/近似）

- `softplus`
- `swish`
- `gelu`（`tanh` 近似）
- `relu`（平滑代理 `relu_soft`）
- `elu`（平滑门控版本）
- `leaky_relu`（平滑门控版本）
- `softsign`（平滑绝对值版本）
- `mish`

### 4.3 Repo-extension 向量模板

- `softmax_template(logits)`
- `logsumexp_template(logits)`
- `cross_entropy_template(logits, target_index)`（one-hot）
- `label_smoothing_cross_entropy_template(logits, target_index, epsilon)`
- `focal_loss_template(logits, target_index, gamma)`
- `focal_loss_template_with_alpha(logits, target_index, gamma, alpha)`
- `batch_softmax_template(batch_logits)`
- `batch_cross_entropy_template(batch_logits, targets)`
- `batch_cross_entropy_mean_template(batch_logits, targets)`
- `batch_label_smoothing_cross_entropy_template(...)`
- `batch_label_smoothing_cross_entropy_mean_template(...)`
- `batch_focal_loss_template(...)`
- `batch_focal_loss_template_with_alpha(...)`
- `batch_focal_loss_mean_template(...)`
- `batch_focal_loss_mean_template_with_alpha(...)`

### 4.4 Repo-extension 自动微分与反降级

- `symbolic_derivative(expr, var_index)`：对 `SourceExpr` 做符号微分
- `simplify_source_expr(expr)`：梯度表达式局部代数简化与常量折叠
- `source_expr_node_count(expr)`：表达式规模统计（用于防止梯度树膨胀）
- `delower_to_source(lowered)`：把纯 EML 树回升为源表达（`exp - log` 形式）
- `raise_expr_to_source(expr)`：把运行时 `Expr` 回升到 `SourceExpr`

## 5. 快速开始

### 5.1 构建与测试

```bash
cargo test
cargo check --all-targets
```

### 5.2 运行示例

```bash
cargo run --example symbolic_regression_loop
cargo run --example pipeline_api
```

### 5.3 CLI 快速检查

```bash
cargo run --bin eml -- parse "exp(x0) - log(x1)"
cargo run --bin eml -- lower "softplus(x0) + sigmoid(x0)"
cargo run --bin eml -- profile "exp(x0) - log(x1)" --sample-count 32
```

更多命令见 `docs/cli.md`。

### 5.5 论文复现与研究 artifact 入口

- 查看 `docs/paper-basis-catalog.md` / `docs/paper-basis-catalog.json` 了解论文基集覆盖状态、witness 与测试锚点。
- 运行 `cargo test --test paper_reproduction` 回放代表性 paper-basis witness。
- 运行 `python scripts/paper_reproduction_summary.py --output-json target/paper-reproduction-summary.json --output-md target/paper-reproduction-summary.md` 生成论文复现摘要 artifact。
- 运行 `python scripts/sr_research_benchmark.py --output-json target/sr-research-benchmark.json --output-md target/sr-research-benchmark.md` 生成符号回归研究 artifact。
- 发布前审阅入口见 `docs/releases/v0.2.0/README.md`。

### 5.4 C ABI 示例

```bash
cargo build --release
```

头文件位于 `include/eml_rs.h`，最小 C 示例位于 `examples/c_ffi_minimal/main.c`。

## 6. 典型工程用法（推荐）

1. 前端表达式（模型/损失/激活）先转 `SourceExpr`。
2. 用 `optimize_for_lowering` 做源级重写与简化。
3. 统一降级为 EML IR（`Expr`）。
4. 在 EML IR 上做 CSE、字节码编译、验证。
5. 部署时根据代价模型选择：
   - 研究模式：直接 EML 执行
   - 生产模式：反降级回平台原生高性能 kernel

## 7. 当前实现中的近似说明

为保证“纯 EML 可执行”，部分函数采用工程近似（尤其训练常见的分段函数）：

- `relu`、`elu`、`leaky_relu` 使用平滑门控表达
- `softsign` 使用平滑绝对值近似
- `gelu` 使用 `tanh` 近似（非 erf 精确版）

这些近似是有意设计，用于可微优化和统一表达。若你需要严格算子语义，可在部署阶段替换为原生精确实现。

## 8. 常见误解

- 误解：EML 主要是“可视化推理过程的基础单位”。  
  事实：可视化只是副产品；核心价值是统一语义、统一编译、统一验证。

- 误解：硬件只要优化 EML 就一定让 AI 训练更快。  
  事实：EML 优化是必要条件之一，不是充分条件。系统级瓶颈（算子代价、并行、带宽、反向图）同样决定最终速度。

## 9. 后续方向

- 收集多轮 nightly artifact，观察 paper reproduction 与 SR research 输出是否稳定。
- 准备正式 `v0.2.0` 前，按 `docs/releases/v0.2.0/verification.md` 运行 release 检查清单。
- 若继续贴近论文发现流程，优先补“最短式 / 搜索 provenance”治理。
- 若继续推进工程集成，优先完善反降级后端与平台互操作。
