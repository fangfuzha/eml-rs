# Developer Guide / 开发者指南

## 中文

### 开发环境

```bash
cargo fmt
cargo test -q
cargo clippy --all-targets -- -D warnings
cargo bench --bench eval_bench
python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json
```

### 论文与工程映射

- 论文给出的是“单二元算子统一表达”的理论可能性。
- 工程实现关注的是四件事：统一表达、统一编译、统一验证、必要时反降级。
- 因此不要把论文结论误读成“EML 直接替代所有原生高性能算子”。
- 进行语义边界、函数覆盖、论文复现相关开发前，优先阅读 `docs/eml-paper-summary.md`，把它当作论文到工程约束的权威入口。
- 在判断某个函数属于论文原始基集还是仓库扩展模板时，查 `docs/paper-basis-catalog.md`，不要只看 README 中的支持函数示例。

### 调试路径

- 数值语义问题：先看 `core` 与 `EvalPolicy`。
- lowering 问题：先看 `eml-lowering` 与 `opt`。
- 运行时不一致：对照 `tree / RPN / bytecode`。
- 性能回退：先跑 `criterion`，再过 `bench_gate.py`。
- 论文复现治理问题：先看 `docs/paper-basis-catalog.json`、`tests/paper_reproduction.rs`、`scripts/paper_reproduction_summary.py`。
- SR 研究 artifact 问题：先看 `scripts/sr_research_benchmark.py` 与 `docs/testing.md` 中记录的 schema / snapping 规则。

### 贡献建议

- 小改动优先补测试，再补文档。
- 新增 pass 或模板时，同时补：语义测试、数值对照、基准覆盖。
- 对公共 API 改动，必须同步更新 `README` 与相关 `docs/*.md`。
- 对错误行为改动，必须同步更新 `EmlErrorCode` / `EmlDiagnostic` 说明。

### 相关文档

- 分层边界见 `docs/architecture.md`
- 验收标准见 `docs/acceptance.md`
- 版本与兼容策略见 `docs/versioning.md`
- 质量矩阵见 `docs/testing.md`
- `v0.2.0` 发布准备草案见 `docs/releases/v0.2.0/README.md`
- 维护与安全响应见 `CONTRIBUTING.md` / `SECURITY.md`

## English

### Development Environment

```bash
cargo fmt
cargo test -q
cargo clippy --all-targets -- -D warnings
cargo bench --bench eval_bench
python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json
```

### Paper-to-engineering Mapping

- The paper establishes the possibility of a single binary operator as a unifying language.
- The implementation focuses on four engineering tasks: unified representation, unified compilation, unified verification, and de-lowering when needed.
- Do not misread the theory as "EML should directly replace every native high-performance kernel."
- Before working on semantic boundaries, function coverage, or paper-fidelity tasks, read `docs/eml-paper-summary.md` first and treat it as the authoritative paper-to-engineering contract.
- When deciding whether a function belongs to the original paper basis or to a repository extension, check `docs/paper-basis-catalog.md` instead of relying on README examples alone.

### Debugging Path

- Numeric semantic issues: start from `core` and `EvalPolicy`.
- Lowering issues: inspect `eml-lowering` and `opt`.
- Runtime mismatches: compare `tree / RPN / bytecode`.
- Performance regressions: run `criterion`, then validate through `bench_gate.py`.
- Paper-reproduction governance issues: start from `docs/paper-basis-catalog.json`, `tests/paper_reproduction.rs`, and `scripts/paper_reproduction_summary.py`.
- SR research artifact issues: inspect `scripts/sr_research_benchmark.py` together with the schema and snapping rules documented in `docs/testing.md`.

### Contribution Guidance

- Prefer adding tests first, then docs, for small changes.
- New passes or templates should ship with semantic tests, numeric comparisons, and benchmark coverage.
- Public API changes must update `README` and the relevant `docs/*.md`.
- Error-behavior changes must update the `EmlErrorCode` / `EmlDiagnostic` documentation.

### Related Docs

- Layer boundaries: `docs/architecture.md`
- Acceptance criteria: `docs/acceptance.md`
- Versioning and compatibility: `docs/versioning.md`
- Quality matrix: `docs/testing.md`
- `v0.2.0` release-preparation draft: `docs/releases/v0.2.0/README.md`
- Maintenance and security response: `CONTRIBUTING.md` / `SECURITY.md`
