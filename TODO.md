# eml-rs TODO / 实现计划

## P0: 最小可用骨架（当前里程碑）
- [x] 建立 `core + ir + verify` 三个基础模块
- [x] 建立 `cargo test` 可运行的对照测试
- [x] 建立 `criterion` benchmark 骨架（tree / rpn / native）
- [x] 增加论文总结文档，明确语义边界和工程约束

## P1: 语义与数值稳定性
- [ ] 明确复对数分支策略（principal / corrected-real）并做成可配置项
- [ ] 加入特殊值语义策略（`inf`, signed zero, `nan`）和一致性测试
- [ ] 增加跨后端对照（`libm` / `mpmath` / `torch.complex128`）

## P2: IR 与编译能力增强
- [ ] 增加 `Expr -> RPN` 以外的字节码执行器（减少解释器开销）
- [ ] 添加从常见表达式到 EML IR 的 lowering（`+,-,*,/,pow,exp,log,sin,cos,...`）
- [ ] 加入 IR 统计（深度、节点数、共享子树数）

## P3: 优化与性能
- [ ] 常量折叠与公共子表达式消除（CSE）
- [ ] 子树重写规则与代价模型（优先减少 `exp/log` 次数）
- [ ] 建立基准回归门禁（CI 中检测性能回退）

## P4: 工程接口与生态
- [ ] 导出 C ABI（`cdylib`）供其他语言调用
- [ ] 提供 `no_std` 评估可行性（若目标包含嵌入式/FPGA 主控）
- [ ] 增加示例应用：符号回归训练环路（EML tree 参数化）
