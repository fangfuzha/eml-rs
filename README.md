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
- `src/ir.rs`: `Expr` IR、RPN、统计信息
- `src/bytecode.rs`: 寄存器字节码执行器（含 CSE/常量折叠）
- `src/opt.rs`: 源表达式重写与代价模型
- `src/verify.rs`: 数值对照验证
- `src/ffi.rs`: C ABI 导出
- `src/lowering.rs`: 对独立 lowering crate 的兼容封装
- `crates/eml-lowering`: `no_std + alloc` 的 parser/lowering 独立 crate
- `tests/`: 集成测试与跨后端验证
- `examples/`: 训练环路、C FFI 最小示例

## 4. 当前支持函数族

### 4.1 初等函数（已实现 lowering）
- `+ - * / pow exp log`
- `sin cos tan`
- `sinh cosh tanh`
- `asin acos atan`
- `sqrt`

### 4.2 AI 常见函数（模板/近似）
- `sigmoid`
- `softplus`
- `swish`
- `gelu`（`tanh` 近似）
- `relu`（平滑代理 `relu_soft`）
- `elu`（平滑门控版本）
- `leaky_relu`（平滑门控版本）
- `softsign`（平滑绝对值版本）
- `mish`

### 4.3 向量模板
- `softmax_template(logits)`
- `logsumexp_template(logits)`
- `cross_entropy_template(logits, target_index)`（one-hot）
- `batch_softmax_template(batch_logits)`
- `batch_cross_entropy_template(batch_logits, targets)`
- `batch_cross_entropy_mean_template(batch_logits, targets)`

### 4.4 自动微分与反降级
- `symbolic_derivative(expr, var_index)`：对 `SourceExpr` 做符号微分
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
```

### 5.3 C ABI 示例

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

- 扩展更多训练模板（label smoothing、focal loss、多任务损失拼接）
- 自动微分进一步工程化（公共子式共享、梯度表达简化、Jacobian/Hessian）
- 反降级器与平台后端对接（直接映射到目标框架算子图）
- 建立 CI 性能回归基线与多平台数值基线
