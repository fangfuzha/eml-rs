# v0.1.1 Migration Notes / 迁移说明

## 中文

从 `v0.1.0` 升级到 `v0.1.1` 时，普通高层 API 使用者通常只需要更新版本号：

```toml
eml-rs = "0.1.1"
```

如果你在 workspace 内部直接依赖分层 crate，也同步更新：

```toml
eml-core = "0.1.1"
eml-lowering = "0.1.1"
```

### 需要注意
- `EvalMetrics` 与 `VerifyMetrics` 增加了 `parallel`、`workers` 信息；如果你用结构体字面量手动构造这些类型，需要补齐字段。
- 并行 batch eval 目前只支持 `BuiltinBackend::Tree` 和 `BuiltinBackend::Rpn`；`BuiltinBackend::Bytecode` 会返回 `EmlError::Unsupported`。
- benchmark 代码应使用 `std::hint::black_box`，不要再依赖 `criterion::black_box`。
- `criterion 0.8.2` 提高了 dev benchmark 依赖链要求；生产构建与 `no_std` 核心层不应依赖它。

## English

For normal high-level API users, upgrading from `v0.1.0` to `v0.1.1` should usually only require a version bump:

```toml
eml-rs = "0.1.1"
```

If you depend on the layered crates directly, update them as well:

```toml
eml-core = "0.1.1"
eml-lowering = "0.1.1"
```

### Notes
- `EvalMetrics` and `VerifyMetrics` now expose `parallel` and `workers`; code that constructs these structs with literals must fill the new fields.
- Parallel batch evaluation currently supports only `BuiltinBackend::Tree` and `BuiltinBackend::Rpn`; `BuiltinBackend::Bytecode` returns `EmlError::Unsupported`.
- Benchmark code should use `std::hint::black_box` instead of `criterion::black_box`.
- `criterion 0.8.2` raises the dev benchmark dependency-chain requirements; production builds and the `no_std` core layer should not depend on it.
