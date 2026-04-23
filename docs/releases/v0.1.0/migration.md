# v0.1.0 Migration Notes / 迁移说明

## 中文

这是首个正式版本快照，因此没有历史版本迁移负担。  
如果你此前直接跟随仓库主线开发，需要注意：

- parser/lowering 逻辑已经固定在独立 crate `eml-lowering`。
- 数值内核已经固定在独立 crate `eml-core`。
- 推荐的新高层入口是 `eml_rs::api::{compile, PipelineBuilder}`。
- 错误处理统一到 `EmlError / EmlErrorCode / EmlDiagnostic`。
- 文档入口已经集中到 `docs/README.md`。

## English

This is the first formal release snapshot, so there is no legacy-version migration burden yet.  
If you were previously following the repository head directly, note:

- Parser/lowering logic now lives in the standalone `eml-lowering` crate.
- Numeric kernels now live in the standalone `eml-core` crate.
- The recommended high-level entry point is `eml_rs::api::{compile, PipelineBuilder}`.
- Error handling is unified through `EmlError / EmlErrorCode / EmlDiagnostic`.
- Documentation entry points are now centralized in `docs/README.md`.
