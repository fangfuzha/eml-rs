# Contributing / 贡献指南

## 中文

### 提交前最少检查
```bash
cargo fmt
cargo test -q
cargo clippy --all-targets -- -D warnings
```

### 变更要求
- 改语义：补测试。
- 改公共 API：补文档。
- 改性能路径：补 benchmark 或 bench gate。
- 改错误行为：同步更新 `EmlErrorCode` 说明。

### PR 期望
- 说明改了什么、为什么改。
- 说明是否影响数值语义、性能基线、C ABI。
- 若有破坏性变更，给出迁移路径。

## English

### Minimum Checks Before Sending A Change
```bash
cargo fmt
cargo test -q
cargo clippy --all-targets -- -D warnings
```

### Change Requirements
- Semantic change: add tests.
- Public API change: update docs.
- Performance-path change: add benchmarks or a bench gate update.
- Error-behavior change: update the `EmlErrorCode` documentation.

### PR Expectations
- Explain what changed and why.
- State whether numeric semantics, performance baselines, or the C ABI are affected.
- Provide a migration path for breaking changes.
