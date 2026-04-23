# Security / 安全策略

## 中文
- 当前 `unsafe` 只允许存在于 FFI 边界。
- `eml-core` 与 `eml-lowering` 禁止 `unsafe` 代码。
- 如果发现会导致崩溃、越界、未定义行为或明显错误传播的缺陷，请按 issue 或私下方式报告。
- 目标：7 天内确认问题是否成立及影响范围。

## English
- `unsafe` is only allowed at the FFI boundary.
- `eml-core` and `eml-lowering` forbid `unsafe` code.
- If you find a crash, out-of-bounds behavior, undefined behavior, or a serious error-propagation issue, report it via issue or private contact.
- Target response: confirm validity and impact within 7 days.
