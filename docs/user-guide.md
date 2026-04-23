# User Guide / 使用者指南

## 中文

### 5 分钟上手

```bash
cargo test
cargo run --example pipeline_api
```

```rust
use eml_rs::api::{compile, BuiltinBackend};

let compiled = compile("sigmoid(x0) + softplus(x0)")?;
let value = compiled.eval_real(BuiltinBackend::Bytecode, &[0.35])?;
println!("{value}");
# Ok::<(), eml_rs::EmlError>(())
```

### 核心概念

| 名称 | 作用 |
| --- | --- |
| `SourceExpr` | 源表达式 AST，适合模板构造、自动微分、源级重写 |
| `LoweredExpr` | standalone lowering crate 的纯 EML 树 |
| `Expr` | runtime IR，适合 tree/RPN/bytecode 执行与统计 |
| `BytecodeProgram` | 对 `Expr` 做 CSE/常量折叠后的寄存器程序 |
| `EvalPolicy` | 复对数分支策略、特殊值策略、近实数阈值 |
| `PipelineBuilder` | 高层开箱即用编译入口 |

### 推荐工作流
1. 用 `PipelineBuilder` 或 `compile()` 建立实验原型。
2. 需要控制 lowering 前重写时，直接操作 `SourceExpr`。
3. 需要研究统一 IR 时，直接操作 `Expr` 与 `BytecodeProgram`。
4. 需要可信对照时，用 `verify` 模块或 `CompiledPipeline::verify_*`。

### 最佳实践
- 研究阶段优先用高层 API，先把语义与验证走通，再下沉到低层模块。
- 对训练模板保持“先表达统一、后平台特化”的思路，不要一开始就把 EML 当最终部署格式。
- 对 `log`、`sqrt`、反三角函数等表达式，显式写清策略和容差。
- 对性能判断不要只看 `mean`，结合 `median` 和 robust `P95/P99` 一起看。

### 常见问题

`Q: 为什么有时 source 表达式可解析，但某些实值样本会在 lowering 后报 domain error？`  
`A:` 纯 EML 展开会引入复数中间量和不同的中间路径，研究阶段要把“最终语义一致”和“中间路径始终实域”区分开。

`Q: 我应该默认用 tree、RPN 还是 bytecode？`  
`A:` 研究默认用 `bytecode` 看执行性能，`tree` 用于语义调试，`RPN` 用于对照 stack-machine 路线。

`Q: 这个项目是不是要替换所有 AI kernel？`  
`A:` 不是。它优先是统一 IR/优化/验证框架，生产部署通常要反降级回原生高性能 kernel。

## English

### 5-minute Quick Start

```bash
cargo test
cargo run --example pipeline_api
```

```rust
use eml_rs::api::{compile, BuiltinBackend};

let compiled = compile("sigmoid(x0) + softplus(x0)")?;
let value = compiled.eval_real(BuiltinBackend::Bytecode, &[0.35])?;
println!("{value}");
# Ok::<(), eml_rs::EmlError>(())
```

### Core Concepts

| Name | Role |
| --- | --- |
| `SourceExpr` | Source AST for templates, autodiff, and source rewrites |
| `LoweredExpr` | Pure EML tree from the standalone lowering crate |
| `Expr` | Runtime IR for tree/RPN/bytecode execution and stats |
| `BytecodeProgram` | Register program after CSE and constant folding |
| `EvalPolicy` | Complex-log branch policy, special-value policy, near-real epsilon |
| `PipelineBuilder` | High-level compile entry point |

### Recommended Workflow
1. Start with `PipelineBuilder` or `compile()` for fast experiments.
2. Drop to `SourceExpr` when you need pre-lowering rewrites.
3. Drop to `Expr` and `BytecodeProgram` when researching unified IR execution.
4. Use `verify` or `CompiledPipeline::verify_*` for trusted comparisons.

### Best Practices
- Prefer the high-level API first, then drop into low-level modules only where the experiment needs it.
- Keep training templates in a "unify first, specialize later" mindset; EML is not automatically the final deployment format.
- Make policies and tolerances explicit for `log`, `sqrt`, and inverse trig expressions.
- Judge performance with `median` and robust `P95/P99`, not only `mean`.

### FAQ

`Q: Why can a source expression parse successfully but still hit a domain error on some real samples after lowering?`  
`A:` Pure EML expansion can introduce complex intermediates and different internal paths. Separate "final semantic equivalence" from "every intermediate stays in the real domain."

`Q: Should I default to tree, RPN, or bytecode?`  
`A:` Use `bytecode` for execution-focused research, `tree` for semantic debugging, and `RPN` as the stack-machine baseline.

`Q: Is this project trying to replace every AI kernel?`  
`A:` No. It is primarily a unified IR/optimization/verification framework. Production deployments should usually de-lower back into native high-performance kernels.
