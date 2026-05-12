# EML 论文核心、工程约束与开发指引

## 中文

### 论文来源

- arXiv: [All elementary functions from a single binary operator (v2)](https://arxiv.org/abs/2603.21852v2)
- 作者代码仓库: [VA00/SymbolicRegressionPackage](https://github.com/VA00/SymbolicRegressionPackage)
- 代码快照: [Zenodo 10.5281/zenodo.19183008](https://doi.org/10.5281/zenodo.19183008)

### EML 核心定义

- 二元算子定义: `eml(x, y) = exp(x) - ln(y)`
- 论文中的统一语法: `S -> 1 | eml(S, S)`
- 编译到带输入变量的工程表达时，可把变量视作额外终端，但“纯 EML 形式”的终端仍只有常数 `1`
- 关键构造示例:
  - `exp(x) = eml(x, 1)`
  - `ln(x) = eml(1, eml(eml(1, x), 1))`

### 论文真正证明了什么

1. 论文证明的是一个具体“科学计算器基集”的完备性，不是对所有函数族的无条件完备性声明。
2. 纯 EML 表达可统一为同构二叉树，这为 IR、枚举、重写、字节码化和硬件映射提供了统一结构。
3. 实函数计算在工程上通常需要复数中间量；负实轴、零点和奇点附近的行为取决于 `log` 分支与特殊值语义。
4. 论文的发现流程与执行流程不是一回事。搜索/发现依赖 bootstrapping 与外部验证，运行时系统则关注 lowering、执行和验证。
5. 论文中的符号回归结果是 proof-of-concept，而不是“深树一定可训练”的工程承诺。盲恢复在深度 `5+` 显著退化。

### 对 eml-rs 的直接工程约束

#### 1) 语义约束先于性能约束

- 任何后端都必须先服从同一数值语义，再讨论优化。
- 因此 `core` 必须独占 `eml` 原子语义、`log` 分支策略和特殊值策略，不能把这些规则散落到 `ir`、`bytecode` 或 FFI 边界。

#### 2) 论文完备性不等于仓库已完整覆盖

- 论文覆盖的是其 Table 1 所定义的科学计算器基集。
- 当前仓库在此基础上实现了常见初等函数 lowering，并额外扩展了 AI 激活/损失模板。
- 这意味着“仓库支持的函数族”分成两类:
  - 论文基集内能力
  - 仓库自行扩展的工程模板

#### 3) 复数内部语义必须显式暴露

- 论文明确指出内部计算需要复数域，并在负实轴附近面临分支符号问题。
- 对应到仓库，`EvalPolicy`、`LogBranchPolicy`、`SpecialValuePolicy` 不是实现细节，而是论文语义的工程化接口。

#### 4) 纯 EML 形式是统一表示，不是默认最优执行形态

- 论文强调统一树结构的好处，但没有声称原样执行总是最快。
- 因此本仓库必须保留 lowering、重写、CSE、字节码、并行策略与反降级路径，而不是把“纯 EML”误当成唯一交付形态。

### 论文主张到代码结构的映射

| 论文主张                     | 工程含义                                       | 当前落点                                                                |
| ---------------------------- | ---------------------------------------------- | ----------------------------------------------------------------------- |
| 单算子统一表达               | 需要统一 IR 与清晰的 lowering 边界             | `crates/eml-lowering`, `src/ir.rs`, `src/lowering.rs`                   |
| 复数中间量不可避免           | 需要显式策略对象与跨后端一致语义               | `crates/eml-core/src/lib.rs`, `src/core.rs`, `src/ffi.rs`               |
| 统一树便于搜索/执行/电路映射 | 需要 tree/RPN/bytecode 多执行形态              | `src/ir.rs`, `src/bytecode.rs`                                          |
| 论文发现依赖外部验证         | 需要独立验证层，而不是把验证混进执行器         | `src/verify.rs`, `tests/reference_compare.rs`, `tests/cross_backend.rs` |
| 统一表示不保证高性能         | 需要 rewrite、基准、门禁、反降级               | `src/opt.rs`, `benches/`, `benchmarks/`, `docs/interoperability.md`     |
| SR 是 proof-of-concept       | 当前只应把 SR 视为研究入口，而不是核心交付承诺 | `examples/symbolic_regression_loop.rs`, `crates/eml-lowering`           |

### 当前项目与论文已经对齐的部分

1. 语义边界清晰。`eml-core` 负责数值内核，根 crate 负责 orchestration，符合“单算子统一语义、分层实现”的要求。
2. 分支策略已显式化。`principal` 与 `corrected-real` 已进入 public policy surface，而不是藏在实现里。
3. 验证层独立存在。tree、RPN、bytecode、参考后端之间的对照测试与采样验证已经成形。
4. 统一表达没有被误读成“直接替代所有 kernel”。`scope`、`developer-guide` 与 `interoperability` 都保留了反降级方向。
5. 论文启发的 SR 相关基础设施已有雏形。当前已有 `symbolic_derivative`、表达式简化、训练模板与示例环路。

### 当前已补齐的“论文级”资产

1. `docs/paper-basis-catalog.json` 已成为机器可读的 paper-basis 事实源，记录 Table 1 能力、覆盖状态、见证式、来源与测试锚点。
2. `tests/paper_reproduction.rs` 已提供轻量 `VerifyBaseSet` 风格 replay，比较 pure EML witness、lowering 结果与 source reference。
3. `scripts/paper_reproduction_summary.py` 已输出 schema v2 摘要，包含覆盖率、missing/partial 明细、witness provenance 与可选严格 gate 参数。
4. `scripts/sr_research_benchmark.py` 已形成独立 SR 研究面，按任务、深度、seed 聚合 recovery、snapping、稳定性与耗时指标。

### 后续仍需审阅的边界

1. paper reproduction 默认仍是 nightly / `workflow_dispatch` artifact；是否升级为主 CI 阻断项，需要等待 artifact 历史稳定后再决定。
2. 当前 witness 治理记录的是可审计构造，不承诺“最短已知 EML 形式”；若要追踪最短式，需要单独定义 provenance 与审阅流程。
3. SR recovery rate 继续作为研究指标，不作为发布阻断条件，也不与 runtime 性能 gate 混合。

### 用这篇论文指导后续开发时的判断规则

1. 新增函数或模板前，先判断它是“论文基集成员”还是“仓库扩展模板”。两者的文档与验收要求应区分。
2. 任何语义改动，先检查负实轴、零点、非有限值与复数中间量，不要先谈性能。
3. 任何优化改动，都不能把“统一表示”误改成“绑定某一后端特化语义”。
4. 任何“更贴近论文”的开发，都应优先补可验证资产，而不是堆更多运行时技巧。

### 建议的下一阶段开发方向

1. 收集若干轮 nightly artifact，观察 paper reproduction 与 SR research 输出是否稳定。
2. 在正式发布 `v0.2.0` 前运行 release 检查清单，并确认 release notes 没有夸大 paper-basis 完备性。
3. 若要继续贴近论文发现流程，优先补“最短式 / 搜索 provenance”治理，而不是把搜索逻辑放进核心 runtime。
4. 若要继续推进工程能力，优先完善反降级后端与平台互操作，保持 paper-basis 与 repo-extension 两条治理链路分离。

### 对“是否要把论文核心提炼成单独 md 文件”的结论

- 结论: 需要，但当前仓库已经有合适承载面，最佳做法不是新增平行摘要，而是把本文件升级为权威入口。
- 原因:
  - 新建第二份摘要容易和现有 `scope`、`architecture`、`developer-guide` 分叉。
  - 论文对项目最有价值的不是“背景介绍”，而是“边界、风险、验证义务、非目标”。
  - 这些内容最适合沉淀在一份开发时会被频繁引用的工程文档中。

## English

### Sources

- arXiv: [All elementary functions from a single binary operator (v2)](https://arxiv.org/abs/2603.21852v2)
- Author repository: [VA00/SymbolicRegressionPackage](https://github.com/VA00/SymbolicRegressionPackage)
- Archival snapshot: [Zenodo 10.5281/zenodo.19183008](https://doi.org/10.5281/zenodo.19183008)

### Core EML definition

- Binary operator: `eml(x, y) = exp(x) - ln(y)`
- Pure EML grammar in the paper: `S -> 1 | eml(S, S)`
- For compiled expressions with variables, inputs can be treated as extra terminals, but the pure EML terminal basis still centers on the constant `1`
- Canonical constructions:
  - `exp(x) = eml(x, 1)`
  - `ln(x) = eml(1, eml(eml(1, x), 1))`

### What the paper actually proves

1. The completeness claim is limited to the paper's concrete scientific-calculator basis, not to arbitrary function families.
2. Pure EML expressions form uniform binary trees, which is valuable for IR design, rewriting, bytecode generation, and hardware mapping.
3. Real-valued math typically requires complex intermediates internally; behavior near the negative real axis and singular points depends on log-branch and special-value policy.
4. The discovery workflow and the execution workflow are different concerns. The paper's search process is not the same as a production lowering/evaluation pipeline.
5. Symbolic-regression results in the paper are proof-of-concept, not a guarantee that deep EML trees are easy to train from random initialization.

### Direct engineering constraints for eml-rs

#### 1) Semantic constraints come before performance constraints

- Every backend must implement the same numeric semantics before any optimization discussion.
- `core` therefore owns `eml`, log-branch policy, and special-value policy instead of scattering those rules across executors.

#### 2) Paper completeness is not the same as repository coverage

- The paper covers its Table 1 scientific-calculator basis.
- This repository implements many of those elementary functions and also adds AI-oriented activation/loss templates.
- As a result, repository functionality splits into:
  - paper-basis capabilities
  - repository-specific engineering extensions

#### 3) Complex-domain semantics must stay explicit

- The paper explicitly relies on complex intermediates and highlights branch-sign issues.
- In this repository, `EvalPolicy`, `LogBranchPolicy`, and `SpecialValuePolicy` are therefore part of the engineering contract, not implementation trivia.

#### 4) Pure EML is a unifying representation, not a guaranteed optimal runtime form

- The paper argues for uniform structure, not for always executing the raw form unchanged.
- The repository must therefore keep lowering, rewriting, CSE, bytecode, parallel policy, and de-lowering as first-class tools.

### Mapping paper claims to repository structure

| Paper claim                                  | Engineering meaning                                                     | Current landing zone                                                    |
| -------------------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Single-operator unification                  | Need a unified IR and explicit lowering boundary                        | `crates/eml-lowering`, `src/ir.rs`, `src/lowering.rs`                   |
| Complex intermediates are unavoidable        | Need policy objects and cross-backend semantic consistency              | `crates/eml-core/src/lib.rs`, `src/core.rs`, `src/ffi.rs`               |
| Uniform trees help search/execution/circuits | Need tree/RPN/bytecode execution forms                                  | `src/ir.rs`, `src/bytecode.rs`                                          |
| Discovery relied on external verification    | Need an explicit verification layer, separate from executors            | `src/verify.rs`, `tests/reference_compare.rs`, `tests/cross_backend.rs` |
| Uniformity does not imply speed              | Need rewrites, benchmarks, gates, and de-lowering                       | `src/opt.rs`, `benches/`, `benchmarks/`, `docs/interoperability.md`     |
| SR is proof-of-concept                       | SR should remain a research-facing track, not a default product promise | `examples/symbolic_regression_loop.rs`, `crates/eml-lowering`           |

### Where the project already aligns with the paper

1. Semantic boundaries are clear: `eml-core` owns numeric meaning and the root crate owns orchestration.
2. Branch policy is explicit and public rather than hidden inside evaluators.
3. Verification is a separate layer with cross-backend and external-reference checks.
4. The repository does not misread the paper as a demand to replace every native kernel.
5. SR-related building blocks already exist, but remain appropriately research-scoped.

### Paper-fidelity assets now in place

1. `docs/paper-basis-catalog.json` is now the machine-readable paper-basis source of truth, covering Table 1 capabilities, coverage status, witnesses, provenance, and test anchors.
2. `tests/paper_reproduction.rs` provides a lightweight `VerifyBaseSet`-style replay comparing pure EML witnesses, lowering output, and source references.
3. `scripts/paper_reproduction_summary.py` emits a schema v2 summary with coverage ratio, missing/partial detail, witness provenance, and optional strict-gate arguments.
4. `scripts/sr_research_benchmark.py` is now a separate SR research surface that aggregates recovery, snapping, stability, and wall-time metrics by task, depth, and seed.

### Remaining review boundaries

1. Paper reproduction remains a nightly / `workflow_dispatch` artifact by default; promotion into a main blocking CI gate should wait for stable artifact history.
2. Current witness governance records auditable constructions, not shortest-known EML forms. Shortest-form tracking needs separate provenance and review rules.
3. SR recovery rate stays a research metric, not a release blocker, and remains separate from runtime performance gates.

### Suggested next development direction

1. Collect several nightly artifact runs and review whether paper reproduction and SR research outputs stay stable.
2. Before a formal `v0.2.0`, run the release checklist and confirm the notes do not overstate paper-basis completeness.
3. If the project moves closer to the paper's discovery workflow, add shortest-form/search provenance governance before adding search logic to the runtime.
4. If the project moves toward engineering integration, prioritize de-lowering backends and platform interop while keeping paper-basis and repo-extension governance separate.

### Decision for future development

- Yes, the paper core should live in a dedicated Markdown document.
- No, this repository does not need a second parallel summary file.
- The better approach is to treat this file as the authoritative paper-to-engineering contract and keep `scope`, `architecture`, and `developer-guide` linked to it.
