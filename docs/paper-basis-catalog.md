# Paper Basis Catalog / 论文基集目录

## 中文

本文件把论文《All elementary functions from a single binary operator》中 Table 1 的 scientific-calculator basis 整理成仓库内可审计资产。

机器可读版本见 `docs/paper-basis-catalog.json`，供后续脚本、测试治理与 future gate 直接消费。

用途：

- 作为 `paper-basis` 与 `repo-extension` 的分界线
- 为后续 completeness harness、见证公式目录、回归测试和 CLI 说明提供统一锚点
- 避免把论文原始能力与仓库扩展模板混写在同一张“支持函数列表”里

### 说明与边界

- 论文的完备性结论针对一个具体的 scientific-calculator basis，而不是任意函数族。
- 本目录优先记录“论文原始基集成员”，不把 softmax、cross-entropy、mish、gelu 等仓库扩展模板混入其中。
- Table 1 的具体表格在 arXiv HTML 渲染中不完整；本目录综合论文正文、作者仓库 `SymbolicRegressionPackage`、以及当前仓库实现状态整理而来。
- 对于少数无法直接逐格抄录确认的条目，本文件会明确标注“高置信还原”。

### 分类约定

- `paper-basis`: 论文 Table 1 基集成员
- `repo-extension`: 仓库自行扩展的模板/近似/训练工程能力，不属于 Table 1 原始基集
- `covered`: 当前仓库已具备明确 AST / lowering / eval 路径
- `partial`: 当前仓库存在相关内部构件，但未形成公开可用的直接成员
- `missing`: 当前仓库尚未形成对应能力入口

## 1. 常量与变量

| 条目     | 类别              | 当前状态  | 证据/说明                                                                                                             | 主要锚点                                           |
| -------- | ----------------- | --------- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| `x`, `y` | `paper-basis`     | `covered` | 高置信还原。论文正文说明以输入变量为起点，验证时用 EulerGamma / Glaisher 代换变量。仓库以 `Var(usize)` 表达输入变量。 | `crates/eml-lowering/src/lib.rs` `SourceExpr::Var` |
| `1`      | `paper-basis`     | `covered` | 论文终端常数。仓库以 `Int(1)` 与 `LoweredExpr::One` 表示。                                                            | `crates/eml-lowering/src/lib.rs`                   |
| `-1`     | `paper-basis`     | `covered` | 可由 `Int(-1)` 直接表达。                                                                                             | `crates/eml-lowering/src/lib.rs`                   |
| `2`      | `paper-basis`     | `covered` | 可由 `Int(2)` 直接表达。                                                                                              | `crates/eml-lowering/src/lib.rs`                   |
| `e`      | `paper-basis`     | `covered` | 有显式 `ConstE`。                                                                                                     | `crates/eml-lowering/src/lib.rs`                   |
| `pi`     | `paper-basis`     | `covered` | 有显式 `ConstPi`。                                                                                                    | `crates/eml-lowering/src/lib.rs`                   |
| `i`      | `paper-basis`     | `covered` | 有显式 `ConstI`。                                                                                                     | `crates/eml-lowering/src/lib.rs`                   |
| `0`      | 非 Table 1 预置项 | `covered` | 论文正文把 `0` 当作可生成常量示例，不视为 Table 1 起始基元。仓库可由 `Int(0)` 表达。                                  | `crates/eml-lowering/src/lib.rs`                   |

## 2. 一元函数

| 条目               | 论文基集 | 当前状态  | 测试锚点                                                                                                                                     | 备注                               |
| ------------------ | -------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| `minus(x)`         | 是       | `covered` | 间接覆盖                                                                                                                                     | 对应 `Neg`                         |
| `half(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_named_composition_entries_match_reference`                         | 命名 parser 入口展开为 `Div(x, 2)` |
| `inv(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_named_composition_entries_match_reference`                         | 命名 parser 入口展开为 `Div(1, x)` |
| `sqr(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_named_composition_entries_match_reference`                         | 命名 parser 入口展开为 `Pow(x, 2)` |
| `sqrt(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Sqrt`                        |
| `exp(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_eml_exp_matches_complex_exp`                                           | 对应 `Exp`                         |
| `log(x)` / `ln(x)` | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_eml_log_formula_matches_real_ln`                                       | 对应 `Log`                         |
| `sin(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_source_lowering_matches_native_reference`                              | 对应 `Sin`                         |
| `cos(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_source_lowering_matches_native_reference`                              | 对应 `Cos`                         |
| `tan(x)`           | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Tan`                         |
| `sinh(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Sinh`                        |
| `cosh(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Cosh`                        |
| `tanh(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Tanh`                        |
| `asin(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Asin`                        |
| `acos(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Acos`                        |
| `atan(x)`          | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_extended_elementary_functions_match_reference`                         | 对应 `Atan`                        |
| `asinh(x)`         | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_inverse_hyperbolic_and_hypot_match_reference`                      | 对应 `Asinh`                       |
| `acosh(x)`         | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_inverse_hyperbolic_and_hypot_match_reference`                      | 对应 `Acosh`                       |
| `atanh(x)`         | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_inverse_hyperbolic_and_hypot_match_reference`                      | 对应 `Atanh`                       |
| `sigmoid(x)`       | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_sigmoid_and_repo_extension_training_family_is_lowerable_and_evaluable` | 论文基集成员，同时也服务训练模板   |

## 3. 二元操作

| 条目          | 论文基集 | 当前状态  | 测试锚点                                                                                                                | 备注                                          |
| ------------- | -------- | --------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `x + y`       | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_source_lowering_matches_native_reference`         | 对应 `Add`                                    |
| `x - y`       | 是       | `covered` | 间接覆盖                                                                                                                | 对应 `Sub`                                    |
| `x * y`       | 是       | `covered` | 间接覆盖                                                                                                                | 对应 `Mul`                                    |
| `x / y`       | 是       | `covered` | 间接覆盖                                                                                                                | 对应 `Div`                                    |
| `pow(x, y)`   | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `bytecode_and_tree_evaluation_agree`                           | 对应 `Pow`；当前测试名仍偏 backend 一致性视角 |
| `log_x(y)`    | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_named_composition_entries_match_reference`    | 命名 parser 入口展开为 `log(y) / log(x)`      |
| `avg(x, y)`   | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_named_composition_entries_match_reference`    | 命名 parser 入口展开为 `(x + y) / 2`          |
| `hypot(x, y)` | 是       | `covered` | [tests/reference_compare.rs](tests/reference_compare.rs) `paper_basis_p22_inverse_hyperbolic_and_hypot_match_reference` | 对应 `Hypot`                                  |

## 4. 仓库扩展模板（非论文原始基集）

下列能力不属于 Table 1 原始基集，但属于当前仓库重要扩展：

- `softplus`
- `swish`
- `gelu_tanh`
- `relu_soft`
- `elu`
- `leaky_relu`
- `softsign`
- `mish`
- `softmax_template`
- `cross_entropy_template`
- `label_smoothing_cross_entropy_template`
- `focal_loss_template`
- batch / mean 版本训练模板
- `symbolic_derivative`
- `portable graph` 导出

这些能力应继续保留，但在文档、测试、CI 和未来 acceptance 中必须与 `paper-basis` 分层治理。

当前测试命名约定：

- `paper_basis_*`: 论文原始基集能力的语义/见证/覆盖测试
- `repo_extension_*`: 仓库扩展模板、训练能力、portable/de-lowering 等扩展测试
- 混合覆盖测试允许在名称中同时出现 `paper_basis` 与 `repo_extension`，前提是测试主体确实横跨两类能力

## 5. 当前缺口总结

优先缺口：

1. `asin` / `acos` / `atan` 仍缺少从公开论文来源直接抄录的显式最短 witness；当前仓库仅声明可执行 lowering witness。
2. 后续 gate 升级仍需统一从机器可读 catalog 消费 replayed witness，避免脚本、测试、文档清单漂移。

## 6. 对后续阶段的直接输入

- P19 completeness harness 应优先覆盖：`exp`、`ln`、`+`、`-`、`*`、`/`、`pow`
- 第二批已补齐：`asinh`、`acosh`、`atanh`、`hypot`
- 文档与测试命名中建议引入前缀：`paper_basis_*` / `repo_extension_*`

## 7. 代表性 EML 见证式与来源

本节收录第一版“可审计见证式”。来源优先级如下：

1. 论文正文或论文 HTML 明确给出的公式
2. 当前仓库 lowering helper 中明确定义的公式
3. 论文确认存在、仓库也有可执行组合式，但不主张其为最短已知形式

除非特别说明，下表中的组合见证式表示“当前仓库采用的可执行 witness”，不等于论文 Table 4 的最短式。

| 能力          | 见证式                               | 来源                                                                          | 置信度   | 备注                                            |
| ------------- | ------------------------------------ | ----------------------------------------------------------------------------- | -------- | ----------------------------------------------- |
| `exp(x)`      | `eml(x, 1)`                          | 论文正文显式给出；仓库 `eml_exp` 直接采用                                     | 明确找到 | 论文最基础显式 witness                          |
| `ln(x)`       | `eml(1, eml(eml(1, x), 1))`          | 论文正文显式给出；论文还给出 RPN `11xE1EE`；仓库 `eml_log` 直接采用           | 明确找到 | 当前最关键的 paper-basis 锚点                   |
| `e`           | `eml(1, 1)`                          | 论文正文与结果段落明确给出；仓库以 `eml_exp(1)` 生成                          | 明确找到 | 常量 witness                                    |
| `-x`          | `(1 - x) - 1`                        | 仓库 `eml_neg` 显式实现                                                       | 明确找到 | 当前 lowering witness；不主张最短               |
| `x - y`       | `eml(ln(x), exp(y))`                 | 由 `eml(x, y) = exp(x) - ln(y)` 直接改写；仓库 `eml_sub` 显式实现             | 明确找到 | 这是最直接的减法 witness 之一                   |
| `x + y`       | `x - (-y)`                           | 仓库 `eml_add` 显式实现                                                       | 明确找到 | 当前 lowering witness；通过 `sub` 与 `neg` 组合 |
| `half(x)`     | `x / 2`                              | 仓库命名 parser 入口展开                                                      | 明确找到 | 组合入口，不新增 AST 变体                       |
| `1 / x`       | `exp(-ln(x))`                        | 仓库 `eml_inv` 显式实现                                                       | 明确找到 | 当前 lowering witness                           |
| `sqr(x)`      | `x * x`                              | 仓库命名 parser 入口展开                                                      | 明确找到 | 组合入口，不新增 AST 变体                       |
| `x * y`       | `exp(ln(x) + ln(y))`                 | 论文引言复述 exp-log 经典恒等式；仓库 `eml_mul` 显式实现                      | 明确找到 | 当前 lowering witness；不主张最短               |
| `x / y`       | `x * (1 / y)`                        | 仓库 `eml_div` 显式实现                                                       | 明确找到 | 当前 lowering witness                           |
| `x ^ y`       | `exp(y * ln(x))`                     | 论文把 `pow` 列入基集；仓库 `eml_pow` 显式实现                                | 明确找到 | 当前 lowering witness；基于经典恒等式           |
| `sqrt(x)`     | `x ^ (1/2)`                          | 仓库 `eml_sqrt` 显式实现                                                      | 明确找到 | 当前 lowering witness                           |
| `sin(x)`      | `(exp(i x) - exp(-i x)) / (2 i)`     | 仓库 `eml_sin` 显式实现；论文引言以 Euler 公式说明三角函数可归入 exp-log 视角 | 明确找到 | 当前 lowering witness；不主张最短               |
| `cos(x)`      | `(exp(i x) + exp(-i x)) / 2`         | 仓库 `eml_cos` 显式实现                                                       | 明确找到 | 当前 lowering witness；不主张最短               |
| `tan(x)`      | `sin(x) / cos(x)`                    | 仓库 `eml_tan` 显式实现                                                       | 明确找到 | 当前 lowering witness                           |
| `sinh(x)`     | `(exp(x) - exp(-x)) / 2`             | 仓库 `eml_sinh` 显式实现                                                      | 明确找到 | 当前 lowering witness                           |
| `cosh(x)`     | `(exp(x) + exp(-x)) / 2`             | 仓库 `eml_cosh` 显式实现                                                      | 明确找到 | 当前 lowering witness                           |
| `tanh(x)`     | `sinh(x) / cosh(x)`                  | 仓库 `eml_tanh` 显式实现                                                      | 明确找到 | 当前 lowering witness                           |
| `asinh(x)`    | `log(x + sqrt(x^2 + 1))`             | 仓库 `eml_asinh` 显式实现                                                     | 明确找到 | 当前 lowering witness                           |
| `acosh(x)`    | `log(x + sqrt(x - 1) * sqrt(x + 1))` | 仓库 `eml_acosh` 显式实现                                                     | 明确找到 | 当前 lowering witness                           |
| `atanh(x)`    | `(log(1 + x) - log(1 - x)) / 2`      | 仓库 `eml_atanh` 显式实现                                                     | 明确找到 | 当前 lowering witness                           |
| `hypot(x, y)` | `sqrt(x^2 + y^2)`                    | 仓库 `eml_hypot` 显式实现                                                     | 明确找到 | 当前 lowering witness                           |
| `log_x(y)`    | `log(y) / log(x)`                    | 仓库命名 parser 入口展开                                                      | 明确找到 | 组合入口，不新增 AST 变体                       |
| `avg(x, y)`   | `(x + y) / 2`                        | 仓库命名 parser 入口展开                                                      | 明确找到 | 组合入口，不新增 AST 变体                       |
| `sigmoid(x)`  | `1 / (1 + exp(-x))`                  | 论文方法段明确把 logistic sigmoid 列为基集成员；仓库 `eml_sigmoid` 显式实现   | 明确找到 | 兼具 paper-basis 与训练模板双重意义             |

### 当前无法在公开来源中直接抄录显式 witness 的条目

- `asin(x)` / `acos(x)` / `atan(x)`：仓库已有 lowering witness，但目前可访问论文正文更偏向“存在性与复杂度”而非直接列式。
- `avg(x, y)` / arbitrary-base `log_x(y)`：当前仓库提供命名 parser 入口并展开为组合表达；不新增独立 AST 变体。

### 对 P19 的直接用途

- 第一批 completeness harness 可直接采用本节的 `exp`、`ln`、`-`、`+`、`*`、`/`、`pow` 见证式
- 三角/双曲函数适合作为第二批见证链，用于验证复数中间量与 branch policy 不会破坏 lowering 语义
- 对于 only-composed witness，回归测试应明确写“当前 lowering witness”，避免误导为“论文最短见证式”

## English

This document turns the paper's Table 1 scientific-calculator basis into an auditable repository asset.

Purpose:

- define the boundary between `paper-basis` and `repo-extension`
- provide a stable anchor for future completeness harnesses, witness catalogs, regression tests, and CLI docs
- avoid mixing the paper's original basis with repository-specific training templates

### Notes and boundary

- The paper's completeness claim is scoped to a concrete scientific-calculator basis.
- This catalog records the original paper-basis members first and keeps softmax, cross-entropy, mish, gelu, and other repository extensions out of that set.
- Because arXiv HTML does not fully render Table 1, this catalog is reconstructed from the paper text, the author's `SymbolicRegressionPackage`, and the current repository implementation.
- Items that cannot be directly transcribed cell-by-cell are marked as high-confidence reconstructions.

### Summary of current status

- Fully covered paper-basis members include: variables, `1/-1/2/e/pi/i`, `neg`, `half`, `inv`, `sqr`, `sqrt`, `exp`, `log`, `sin/cos/tan`, `sinh/cosh/tanh`, `asin/acos/atan`, `asinh/acosh/atanh`, `sigmoid`, the basic arithmetic operators, `pow`, arbitrary-base `log`, `avg`, and `hypot`.
- Composition-only paper-basis names such as `half`, `inv`, `sqr`, `avg`, and arbitrary-base `log` are exposed through named parser entries that expand to existing AST combinations instead of dedicated variants.
- Missing paper-basis members currently include: none as direct public source/lowering entries.

### Representative witness formulas and sources

This first witness set records auditable formulas using three source tiers:

1. formulas stated explicitly in the paper
2. formulas implemented explicitly in the current lowering helpers
3. formulas whose existence is confirmed by the paper, while the repository records an executable compositional witness without claiming shortest-known status

| Capability    | Witness                              | Source                                                                                | Confidence | Note                                     |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | ---------- | ---------------------------------------- |
| `exp(x)`      | `eml(x, 1)`                          | explicit in paper; directly used by `eml_exp`                                         | explicit   | canonical paper witness                  |
| `ln(x)`       | `eml(1, eml(eml(1, x), 1))`          | explicit in paper; paper also gives RPN `11xE1EE`; directly used by `eml_log`         | explicit   | primary paper-basis anchor               |
| `e`           | `eml(1, 1)`                          | explicit in paper/results; repository derives it via `eml_exp(1)`                     | explicit   | constant witness                         |
| `-x`          | `(1 - x) - 1`                        | explicit current lowering helper `eml_neg`                                            | explicit   | repository witness, not claimed shortest |
| `x - y`       | `eml(ln(x), exp(y))`                 | direct algebraic rewrite of the EML definition; explicit helper `eml_sub`             | explicit   | direct subtraction witness               |
| `x + y`       | `x - (-y)`                           | explicit helper `eml_add`                                                             | explicit   | repository witness                       |
| `half(x)`     | `x / 2`                              | named parser entry                                                                    | explicit   | composition entry, no new AST variant    |
| `1 / x`       | `exp(-ln(x))`                        | explicit helper `eml_inv`                                                             | explicit   | repository witness                       |
| `sqr(x)`      | `x * x`                              | named parser entry                                                                    | explicit   | composition entry, no new AST variant    |
| `x * y`       | `exp(ln(x) + ln(y))`                 | classical exp-log identity echoed in paper introduction; explicit helper `eml_mul`    | explicit   | repository witness, not claimed shortest |
| `x / y`       | `x * (1 / y)`                        | explicit helper `eml_div`                                                             | explicit   | repository witness                       |
| `x ^ y`       | `exp(y * ln(x))`                     | `pow` is part of the paper basis; explicit helper `eml_pow`                           | explicit   | repository witness                       |
| `sqrt(x)`     | `x ^ (1/2)`                          | explicit helper `eml_sqrt`                                                            | explicit   | repository witness                       |
| `sin(x)`      | `(exp(i x) - exp(-i x)) / (2 i)`     | explicit helper `eml_sin`; paper introduction motivates trig via Euler/exp-log        | explicit   | repository witness, not claimed shortest |
| `cos(x)`      | `(exp(i x) + exp(-i x)) / 2`         | explicit helper `eml_cos`                                                             | explicit   | repository witness                       |
| `tan(x)`      | `sin(x) / cos(x)`                    | explicit helper `eml_tan`                                                             | explicit   | repository witness                       |
| `sinh(x)`     | `(exp(x) - exp(-x)) / 2`             | explicit helper `eml_sinh`                                                            | explicit   | repository witness                       |
| `cosh(x)`     | `(exp(x) + exp(-x)) / 2`             | explicit helper `eml_cosh`                                                            | explicit   | repository witness                       |
| `tanh(x)`     | `sinh(x) / cosh(x)`                  | explicit helper `eml_tanh`                                                            | explicit   | repository witness                       |
| `asinh(x)`    | `log(x + sqrt(x^2 + 1))`             | explicit helper `eml_asinh`                                                           | explicit   | repository witness                       |
| `acosh(x)`    | `log(x + sqrt(x - 1) * sqrt(x + 1))` | explicit helper `eml_acosh`                                                           | explicit   | repository witness                       |
| `atanh(x)`    | `(log(1 + x) - log(1 - x)) / 2`      | explicit helper `eml_atanh`                                                           | explicit   | repository witness                       |
| `hypot(x, y)` | `sqrt(x^2 + y^2)`                    | explicit helper `eml_hypot`                                                           | explicit   | repository witness                       |
| `log_x(y)`    | `log(y) / log(x)`                    | named parser entry                                                                    | explicit   | composition entry, no new AST variant    |
| `avg(x, y)`   | `(x + y) / 2`                        | named parser entry                                                                    | explicit   | composition entry, no new AST variant    |
| `sigmoid(x)`  | `1 / (1 + exp(-x))`                  | logistic sigmoid is explicitly part of the paper basis; explicit helper `eml_sigmoid` | explicit   | both paper-basis and training-relevant   |

### Items whose explicit public witnesses are still not transcribed here

- `asin(x)` / `acos(x)` / `atan(x)`: the repository has lowering witnesses, but the paper material currently available to this workflow is stronger on existence and complexity than on directly printed formulas.
- `avg(x, y)` and arbitrary-base `log_x(y)`: surfaced as named parser entries that expand to existing AST combinations rather than dedicated variants.

### Repository extensions outside the original paper basis

Repository extensions include `softplus`, `swish`, `gelu_tanh`, `relu_soft`, `elu`, `leaky_relu`, `softsign`, `mish`, vector/batch training templates, symbolic differentiation, and portable-graph export.

These remain important features, but they should be governed separately from `paper-basis` coverage in docs, tests, CI, and future acceptance criteria.

Machine-readable export: `docs/paper-basis-catalog.json`.
