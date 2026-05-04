# eml-rs TODO / 实现计划

## P0: 最小可用骨架（当前里程碑）
- [x] 建立 `core + ir + verify` 三个基础模块
- [x] 建立 `cargo test` 可运行的对照测试
- [x] 建立 `criterion` benchmark 骨架（tree / rpn / native）
- [x] 增加论文总结文档，明确语义边界和工程约束

## P1: 语义与数值稳定性
- [x] 明确复对数分支策略（principal / corrected-real）并做成可配置项
- [x] 加入特殊值语义策略（`inf`, signed zero, `nan`）和一致性测试
- [x] 增加跨后端对照（`libm` / `mpmath` / `torch.complex128`）

## P2: IR 与编译能力增强
- [x] 增加 `Expr -> RPN` 以外的字节码执行器（减少解释器开销）
- [x] 添加从常见表达式到 EML IR 的 lowering（`+,-,*,/,pow,exp,log,sin,cos,...`）
- [x] 加入 IR 统计（深度、节点数、共享子树数）

## P3: 优化与性能
- [x] 常量折叠与公共子表达式消除（CSE）
- [x] 子树重写规则与代价模型（优先减少 `exp/log` 次数）
- [x] 建立基准回归门禁（CI 中检测性能回退）

## P4: 工程接口与生态
- [x] 导出 C ABI（`cdylib`）供其他语言调用
- [x] 提供 `no_std` 评估可行性（若目标包含嵌入式/FPGA 主控）
- [x] 增加示例应用：符号回归训练环路（EML tree 参数化）

## P5: 训练工程能力增强
- [x] 增加 SourceExpr 自动微分（symbolic derivative）与梯度对照测试
- [x] 自动微分表达式简化（局部代数化简 + 常量折叠）以抑制梯度树膨胀
- [x] 增加 batch 级 softmax / cross_entropy / mean-CE 模板
- [x] 增加 label smoothing / focal loss（含 batch + mean 模板）
- [x] 增加 de-lowering（EML IR -> source）与语义一致性测试
- [x] 扩展 benchmark 与 gate（新增 softmax-CE bytecode/tree 比值规则）

## P6: 顶层前提冻结（必须先完成）
- [x] 决策 D1：项目核心能力定位 = `统一 IR/编译优化框架`
- [x] 决策 D2：核心场景优先级 = `研究实验优先`
- [x] 决策 D3：目标用户画像主排序 = `算法研究员 > 业务开发 > 嵌入式开发`
- [x] 决策 D4：非目标清单冻结（v1）
- [x] D4-NG1：不做分布式训练（多机并行/参数服务器）
- [x] D4-NG2：不做在线服务框架（HTTP/gRPC 推理服务）
- [x] D4-NG3：不做 GUI 平台（仅库 + 文档）
- [x] D4-NG4：不做自研 GPU Kernel 运行时（GPU 仅走反降级接外部框架）
- [x] D4-NG5：不做全语言绑定（v1 仅 Rust + C ABI）
- [x] D4-NG6：不做模型 Zoo / 全训练脚本生态
- [x] D4-NG7：不做强实时承诺
- [x] D4-NG8：1.0 前不承诺 API 完全稳定（但保留迁移说明）
- [x] 决策 D5：竞品与对标基线冻结
- [x] D5-B1：能力对标 `MLIR/TVM`（IR 分层、重写、lowering、扩展性）
- [x] D5-B2：数值参考 `PyTorch/NumPy`
- [x] D5-B3：内部性能基线 `tree evaluator`
- [x] D5-B4：`bytecode/tree <= 1.00`，目标 `<= 0.85`
- [x] D5-B5：`RPN/tree <= 1.10`
- [x] D5-B6：导数简化后节点数 `<= naive 的 60%`
- [x] D5-B7：CE/LabelSmoothing/Focal lowering 误差 `<= 1e-5`
- [x] 产出《范围声明》：输入、输出、核心场景、非目标、里程碑边界（`docs/scope.md`）

## P7: 量化目标与验收门槛（无量化不开发）
- [x] 决策 D6：性能目标（已冻结，研究优先）
- [x] D6-P1：`verify+lowering` 在 `10k nodes` 目标下 `P99 <= 25ms`
- [x] D6-P2：`bytecode/tree` 耗时比 `median <= 0.85`，`P99 <= 0.95`
- [x] D6-P3：`1M nodes` 峰值内存 `<= 1.0GB`
- [x] D6-P4：冷启动目标 `<= 800ms`
- [x] 决策 D7：优化优先级（已冻结）
- [x] D7-O1：`CPU 单核执行效率`
- [x] D7-O2：`编译期优化（lowering/rewrite）`
- [x] D7-O3：`内存占用`
- [x] D7-O4：`CPU 多核并行`
- [x] D7-O5：`GPU 适配`
- [x] 决策 D8：资源约束（已冻结）
- [x] D8-R1：默认线程数 `min(num_cpus, 8)`，线程上限 `16`
- [x] D8-R2：基准机内存上限 `4GB`，CI 内存预算 `2GB`
- [x] D8-R3：维持 `core no_std+alloc` 分层，整体不强制无堆模式
- [x] 决策 D9：基准环境（已冻结）
- [x] D9-B1：主基准平台 `x86_64 Linux (8C16T/32GB)`
- [x] D9-B2：数据规模覆盖 `1k/10k/100k nodes` 与 `batch 32/256/1024`
- [x] D9-B3：统一 `criterion` 参数（固定 warmup + measurement）
- [x] D9-B4：统一输出 `median/P95/P99 + RSS + bytecode/tree ratio`
- [x] D9-B5：性能回退阈值 `>5%` 时阻断合并
- [x] 建立验收表（功能/性能/质量/文档/交付）并写入 `docs/acceptance.md`
- [x] CI 门禁化：不达标阻断合并（性能回退阈值、覆盖率阈值、兼容性阈值）

## P8: 架构与接口治理
- [x] 输出分层架构文档 `docs/architecture.md`（算法层/平台抽象层/API 层/绑定层/工具层）
- [x] 冻结跨层调用规则（禁止绕层访问，新增 lint/审查规则）
- [x] 插件化扩展点定义（`src/plugin.rs`：`SourcePass / ExprPass / ExecutionBackend / PipelineObserver`）
- [x] 错误处理体系统一（`src/error.rs`：错误类型、错误码、诊断信息）
- [x] API 分层冻结（`src/api.rs` 高层开箱即用 API + 低层可组合 API）
- [x] 稳定性承诺（`docs/versioning.md`：SemVer、弃用流程、MSRV、兼容策略）
- [x] 安全约束（根 crate `deny(unsafe_op_in_unsafe_fn)`；`eml-core/eml-lowering` `forbid(unsafe_code)`）

## P9: 工程质量与自动化
- [x] 单元测试覆盖率阈值（CI `cargo llvm-cov`：行 `>=80%`，region `>=70%`）
- [x] 集成测试矩阵（`docs/testing.md` + `tests/*.rs`）
- [x] 兼容性矩阵（CI：Windows/Linux/macOS + `x86_64/aarch64` target check + MSRV）
- [x] Fuzzing（`fuzz/` 目标 + `nightly.yml`）
- [x] 性能回归自动化（每次 PR 跑基准并比对基线）
- [x] 依赖安全扫描与许可证合规（`cargo audit` + `cargo deny` + `deny.toml`）
- [x] CI/CD 全流程（`ci.yml` + `release.yml` + `nightly.yml`）

## P10: 文档与交付
- [x] 使用者文档完善：快速上手、概念手册、API 参考、最佳实践、FAQ（`docs/user-guide.md`）
- [x] 开发者文档完善：架构、原理映射、开发指南、调试手册、贡献流程（`docs/developer-guide.md`, `CONTRIBUTING.md`）
- [x] 文档版本化（`docs/versioning.md` + `docs/releases/README.md`）
- [x] 决策 D10：文档语言策略（已冻结）
- [x] D10-L1：中英双语全量同步（用户文档与开发者文档同步维护）
- [x] 决策 D11：交付形态优先级（已冻结）
- [x] D11-P1：`Rust crate > C ABI > CLI > Python 绑定(实验) > 服务化(暂不做)`
- [x] 可观测性方案（`src/api.rs` observer/report + `docs/observability.md`）

## P11: 版本治理与生态
- [x] 决策 D12：发布节奏（已冻结）
- [x] D12-R1：稳定版每 4 周发布一次
- [x] D12-R2：bugfix 按需每周发布
- [x] D12-R3：1.0 前不提供 LTS
- [x] D12-R4：1.0 后每 6 个月一个 LTS，维护周期 12 个月
- [x] D12-R5：项目为个人维护，发布节奏与维护承诺以维护者时间与兴趣为准，可暂停开发
- [x] 维护策略（`docs/maintenance.md` + `SECURITY.md`）
- [x] 生态集成路线（`docs/ecosystem.md`）
- [x] 合规声明（`docs/compliance.md` + `LICENSE`）

## P12: `v0.1.1` 正式发布
- [x] 确认 `main` 工作区干净，`v0.1.1` tag 尚未存在
- [x] 推送 `v0.1.1` tag 到远端
- [x] 等待 `release.yml` 在 Linux / Windows / macOS 上通过
- [x] 下载 release workflow artifacts，并压缩为 GitHub Release assets
- [x] 创建 GitHub Release `v0.1.1`，使用 `docs/releases/v0.1.1/release-notes.md` 作为说明
- [x] 验证 release 页面、tag、assets、CI 状态可访问
- [x] 修正 `release.yml`，后续 tag push 自动创建 GitHub Release 并上传 assets

## P13: 指标自动化补齐
- [x] 新增内存 RSS 采集脚本，覆盖 `1M nodes <= 1.0GB` 的人工验收项
- [x] 新增冷启动采集脚本，覆盖 `cold start <= 800ms` 的人工验收项
- [x] 将 RSS / cold-start 结果写入机器可读 JSON，便于后续 CI gate 使用
- [x] 更新 `docs/acceptance.md`，把可自动化部分从 `manual audit` 切到 `CI/manual tool enforced`
- [x] 在 nightly 或 workflow_dispatch 中加入轻量指标任务，避免每次 push 都跑重负载

## P14: CLI MVP
- [x] 新增 `eml` CLI crate 或根 crate binary，保持库 API 不被 CLI 反向污染
- [x] 支持 `eml parse`：源表达式到结构化 `SourceExpr` 输出
- [x] 支持 `eml lower`：源表达式到 EML IR / stats 输出
- [x] 支持 `eml verify`：输入表达式与样本 JSON，执行 tree/RPN/bytecode 对照
- [x] 支持 `eml profile`：输出 lowering/simplify/bytecode/eval 分段耗时
- [x] 增加 CLI 示例文档与端到端测试

## P15: API 稳定化
- [x] 梳理 public API，明确稳定入口、实验入口和内部入口
- [x] 为核心 public struct / enum / function 补齐 rustdoc 示例
- [x] 增加 `cargo doc` 检查，避免公开 API 文档持续退化
- [x] 建立 `deprecated` 流程示例，避免 0.x 阶段接口演进失控
- [x] 更新 `docs/versioning.md` 与 `docs/user-guide.md`，对齐实际 API 分层

## P16: 反降级与生态互操作
- [x] 定义 portable graph JSON，作为 `SourceExpr / Expr` 对外交换格式
- [x] 实现 `SourceExpr -> portable graph JSON` 导出
- [x] 实现 `Expr -> portable graph JSON` 导出，保留 EML 节点语义
- [x] 增加 PyTorch/NumPy 对照脚本入口，作为研究验证后端
- [x] 增加文档说明：EML IR 如何反降级到目标框架算子图

## P17: 性能第二轮
- [ ] 基于 profiling 数据补齐 `100k nodes` 与更大 batch 的基准覆盖
- [ ] 针对 bytecode eval 做第二轮单核热点优化，先压低单样本成本
- [ ] 对 Tree/RPN 样本级并行做阈值调优，避免小 batch 并行反而变慢
- [ ] 评估 Bytecode batch 是否值得并行化，要求先有 benchmark 证据
- [ ] 将新的性能门槛写入 `benchmarks/gate.json`，避免优化回退

## 决策清单（需要你确认后执行）
- [x] D1 核心能力定位：`统一 IR/编译优化框架`
- [x] D2 核心场景优先级：`研究实验优先`
- [x] D3 目标用户主排序：`算法研究员 > 业务开发 > 嵌入式`
- [x] D4 非目标清单：`v1 非目标 8 条已冻结`
- [x] D5 竞品与基线：`MLIR/TVM + PyTorch/NumPy + 内部 tree 基线`
- [x] D6 性能目标：`P99<=25ms, bytecode/tree<=0.85(median), 1M nodes<=1.0GB, cold<=800ms`
- [x] D7 优化优先级：`CPU 单核 > 编译期优化 > 内存 > CPU 多核 > GPU`
- [x] D8 资源约束：`threads=min(num_cpus,8), max=16, mem=4GB(CI=2GB), core no_std+alloc`
- [x] D9 基准环境：`x86_64 Linux(8C16T/32GB), 1k/10k/100k + batch32/256/1024, regression>5% block`
- [x] D10 文档语言策略：`中英双语全量同步`
- [x] D11 交付形态优先级：`Rust crate > C ABI > CLI > Python 绑定(实验) > 服务化(暂不做)`
- [x] D12 发布与 LTS 策略：`4周稳定版 + 每周按需bugfix + 1.0后6个月LTS(维护12个月) + 个人项目可暂停`
