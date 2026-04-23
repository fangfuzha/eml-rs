# EML 论文要点与工程指导

## 论文链接
- arXiv: [All elementary functions from a single binary operator (v2)](https://arxiv.org/abs/2603.21852v2)
- 作者代码仓库: [VA00/SymbolicRegressionPackage](https://github.com/VA00/SymbolicRegressionPackage)
- 代码快照: [Zenodo 10.5281/zenodo.19183008](https://doi.org/10.5281/zenodo.19183008)

## EML 核心定义
- 二元算子定义: `eml(x, y) = exp(x) - ln(y)`
- 论文中给出的统一语法（以常数 `1` 为终端）: `S -> 1 | eml(S, S)`
- 关键示例:
  - `exp(x) = eml(x, 1)`
  - `ln(x) = eml(1, eml(eml(1, x), 1))`

## 论文核心结论（用于工程决策）
1. 在论文定义的“科学计算器基集”范围内，`eml + 常数 1` 可构造常见初等函数。
2. 表达式可统一成同构二叉树，这对 IR 设计、编译和硬件映射非常有利。
3. 计算需要复数中间量，且 `log` 分支选择会影响实轴行为，工程上必须显式定义策略。
4. 纯 EML 形式具备表示统一性，但不等于执行最优；实际部署需要优化与重写。

## 对 eml-rs 的落地指导

### 1) 模块边界建议
- `core`: 原子算子 `eml` 与数值语义（domain、non-finite、分支策略）。
- `ir`: 统一表达式树 `One / Var / Eml`，并提供 RPN 或字节码执行路径。
- `verify`: 基于采样点的对照验证（与标准函数或外部后端比对）。

### 2) 验证优先级
- 先验证基础恒等式（`exp`, `ln`）再扩展到组合表达式。
- 样本需要覆盖:
  - 正实轴（基础正确性）
  - 负实轴及零邻域（分支与奇点行为）
  - 复平面随机点（内部一致性）

### 3) 性能路线
- 第一阶段: 树解释执行 + RPN 栈执行，对比 native baseline。
- 第二阶段: 常量折叠、CSE、子树重写降低 `exp/log` 调用数。
- 第三阶段: 按目标后端（CPU/GPU/电路）做 lowering。

### 4) 风险与边界
- 论文结论针对特定基集，不应外推为“所有函数族自动可表达”。
- `log` 分支与特殊浮点值处理（`inf/nan/signed zero`）不统一会导致跨平台差异。
- 对实函数结果的断言应包含 `imag_tolerance`，避免误判。
