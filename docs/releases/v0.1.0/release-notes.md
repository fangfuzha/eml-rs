# v0.1.0 Release Notes / 发布说明

## 中文

`v0.1.0` 是 `eml-rs` 的首个发布快照，定位为研究优先的统一 IR / 编译优化框架基础版。

### 本版重点
- 完成 `eml-core` 与 `eml-lowering` 的分层拆分，支持 `no_std + alloc` 的核心分层。
- 提供 `Expr`、RPN、bytecode 三套执行路径，以及 CSE/常量折叠优化。
- 覆盖常见初等函数、常见 AI 激活/损失模板、batch softmax / cross-entropy / label smoothing / focal loss。
- 提供符号微分、表达式简化、反降级、跨后端验证。
- 提供高层 pipeline API、插件扩展点、统一错误码与诊断。
- 提供 C ABI、最小 C 示例、benchmark gate、跨平台 CI、nightly/fuzz/release workflow。

### 当前边界
- 目标仍然是统一 IR / 优化 / 验证，而不是替代所有原生高性能 kernel。
- GPU 运行时、服务化框架、分布式训练、全语言绑定都不在本版本范围内。

## English

`v0.1.0` is the first release snapshot of `eml-rs`, positioned as a research-first unified IR and compiler-optimization framework baseline.

### Highlights
- Completed the `eml-core` / `eml-lowering` split with a `no_std + alloc`-friendly core layering.
- Shipped `Expr`, RPN, and bytecode execution backends with CSE and constant folding.
- Covered common elementary functions, common AI activations/loss templates, and batch softmax / cross-entropy / label smoothing / focal loss.
- Added symbolic differentiation, expression simplification, de-lowering, and cross-backend verification.
- Added a high-level pipeline API, plugin extension points, and unified error codes/diagnostics.
- Added a C ABI, minimal C example, benchmark gates, cross-platform CI, and nightly/fuzz/release workflows.

### Current Boundary
- The target remains unified IR / optimization / verification, not replacement of every native high-performance kernel.
- GPU runtimes, serving frameworks, distributed training, and full language-binding coverage are out of scope for this release.
