# Scope / 范围声明

## 中文

### 项目定位
`eml-rs` 是一个面向研究实验的统一 IR / 编译优化框架。  
它的核心任务不是替换所有高性能算子实现，而是把异构初等函数、训练模板和损失表达统一降到 EML IR，再在统一语义上完成优化、验证、执行和反降级。

### 输入与输出

| 项目 | 说明 |
| --- | --- |
| 输入 | `SourceExpr`、模板化激活/损失表达、向量级 softmax / cross-entropy 家族、以及直接构造的 `Expr` |
| 中间表示 | `LoweredExpr`（standalone lowering tree）与 `Expr`（runtime IR） |
| 输出 | 树执行、RPN、字节码执行结果；验证报告；C ABI 可调用入口；反降级后的 `SourceExpr` |

### 核心场景

| 优先级 | 场景 | 目标 |
| --- | --- | --- |
| P0 | 研究实验 | 快速表达、降级、重写、验证新函数族和训练模板 |
| P1 | 算法原型 | 比较 tree / RPN / bytecode 的数值与性能行为 |
| P2 | 嵌入集成 | 通过 Rust crate 或 C ABI 嵌入其他系统 |
| P3 | 生产部署前准备 | 为后续反降级到原生 kernel 提供统一 IR 和验证基线 |

### 目标用户

| 排序 | 用户 | 关注点 |
| --- | --- | --- |
| 1 | 算法研究员 | 表达能力、重写能力、验证能力、可快速试验 |
| 2 | 业务开发工程师 | 可复用 API、稳定输入输出、C ABI 接口 |
| 3 | 嵌入式开发者 | `no_std + alloc` 分层、较小依赖面 |

### v1 非目标
- 不做分布式训练或参数服务器。
- 不做在线服务框架（HTTP / gRPC 推理服务）。
- 不做 GUI 平台。
- 不做自研 GPU kernel 运行时。
- 不做全语言绑定，当前只保证 Rust crate 与 C ABI。
- 不做模型 Zoo 或完整训练脚本生态。
- 不做强实时承诺。
- 1.0 前不承诺 API 完全稳定。

### 当前里程碑边界

| 在范围内 | 暂不在范围内 |
| --- | --- |
| parser / lowering 独立 crate | 服务化部署 |
| 统一 EML IR + tree / RPN / bytecode | 分布式训练 |
| 数值验证、跨后端对照、自动微分与简化 | 自研 GPU runtime |
| AI 常见激活与损失模板 | GUI 与模型管理平台 |
| Rust crate、C ABI、最小 C 示例 | 全量 Python/Go/Java 绑定 |

## English

### Project Positioning
`eml-rs` is a research-first unified IR and compiler optimization framework.  
Its goal is not to replace every high-performance kernel, but to lower heterogeneous elementary functions, training templates, and loss expressions into one EML IR, then optimize, verify, execute, and de-lower on top of that shared semantics.

### Inputs and Outputs

| Item | Description |
| --- | --- |
| Inputs | `SourceExpr`, templated activations/losses, vector-level softmax / cross-entropy families, and directly built `Expr` trees |
| Intermediate representations | `LoweredExpr` (standalone lowering tree) and `Expr` (runtime IR) |
| Outputs | Tree/RPN/bytecode execution results, verification reports, C ABI entry points, and de-lowered `SourceExpr` |

### Core Scenarios

| Priority | Scenario | Goal |
| --- | --- | --- |
| P0 | Research experiments | Rapidly express, lower, rewrite, and verify new function families and training templates |
| P1 | Algorithm prototyping | Compare numeric and performance behavior across tree / RPN / bytecode backends |
| P2 | Embedded integration | Reuse the library via Rust crate or C ABI |
| P3 | Pre-production preparation | Provide a unified IR and verification baseline before de-lowering back to native kernels |

### Target Users

| Rank | User | Primary concern |
| --- | --- | --- |
| 1 | Algorithm researchers | Expressiveness, rewrites, verification, fast iteration |
| 2 | Application engineers | Reusable APIs, stable I/O shape, C ABI entry points |
| 3 | Embedded developers | `no_std + alloc` layering and small dependency surface |

### v1 Non-goals
- No distributed training or parameter-server support.
- No online serving framework (HTTP / gRPC inference service).
- No GUI platform.
- No custom GPU kernel runtime.
- No full language-binding matrix; only Rust crate and C ABI are promised now.
- No model zoo or full training-script ecosystem.
- No hard real-time guarantee.
- No full API stability promise before `1.0`.

### Current Milestone Boundary

| In scope | Out of scope for now |
| --- | --- |
| Parser/lowering as a standalone crate | Service deployment |
| Unified EML IR with tree / RPN / bytecode execution | Distributed training |
| Numeric verification, cross-backend checks, autodiff, and simplification | Custom GPU runtime |
| AI-oriented activation and loss templates | GUI and model-management platform |
| Rust crate, C ABI, and a minimal C example | Full Python/Go/Java bindings |
