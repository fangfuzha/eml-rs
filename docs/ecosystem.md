# Ecosystem Roadmap / 生态集成路线

## 中文

### 优先级
1. `PyTorch/NumPy`: 继续作为数值参考与验证后端。
2. `MLIR/TVM`: 作为 IR/编译器能力对标对象，而不是直接替换目标。
3. `C ABI` 宿主系统：作为多语言或嵌入式系统的最小集成面。
4. Python 绑定：实验性路线，优先在 Rust crate 稳定后推进。

### 暂不做
- 服务化框架
- 自研 GPU runtime
- 全语言绑定矩阵

## English

### Priority
1. `PyTorch/NumPy`: continue as numeric reference backends.
2. `MLIR/TVM`: remain the capability benchmark for IR/compiler design, not a direct replacement target.
3. `C ABI` host systems: the minimal integration surface for multi-language or embedded adoption.
4. Python bindings: experimental path, after the Rust crate surface is more stable.

### Not Planned For Now
- Serving framework
- Custom GPU runtime
- Full multi-language binding matrix
