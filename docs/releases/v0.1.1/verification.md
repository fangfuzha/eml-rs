# v0.1.1 Verification Record / 发布前验证记录

## 中文

### 本地验证
- `cargo check --workspace` 通过，并同步验证 `eml-rs`, `eml-core`, `eml-lowering` 的 `0.1.1` 构建面。
- `cargo fmt --check` 通过。
- `cargo test -q` 通过。
- `cargo clippy --all-targets -- -D warnings` 通过。
- `cargo bench --bench eval_bench --no-run` 通过。
- 说明：本机未安装 `x86_64-unknown-linux-gnu` Rust target，因此 `cargo check --workspace --target x86_64-unknown-linux-gnu` 的本地失败属于环境缺失；远端 CI 已安装目标并覆盖该路径。

### 远端 PR 验证
- PR: `https://github.com/fangfuzha/eml-rs/pull/3`
- 合并提交: `12d1413`
- PR CI run: `24891283888`, `24891285458`
- 结果: `changes`, `test`, `lint`, `no_std`, `coverage`, `supply-chain`, `compat` 全部通过。
- 后续 main CI run: `24892142360`，验证 `dorny/paths-filter@v4.0.1` 后 `Node.js 20` runtime annotation 已消失。

### 手动 nightly bench-only
- Run: `https://github.com/fangfuzha/eml-rs/actions/runs/24891397303`
- Job: `nightly-bench` (`72884405360`)
- 结果: 通过，耗时约 `3m58s`。
- benchmark gate:
  - `rpn_vs_tree_median = 1.0230`，上限 `1.1500`
  - `bytecode_vs_tree_median = 0.1693`，上限 `0.8500`
  - `bytecode_vs_tree_p99 = 0.1697`，上限 `0.9500`
  - `softmax_ce_bytecode_vs_tree_median_batch1024 = 0.7369`，上限 `0.8500`
  - `softmax_ce_bytecode_vs_tree_p99_batch1024 = 0.7417`，上限 `0.9500`
  - `lower_verify_10k_nodes_p99 = 1111634.45 ns/iter`，上限 `25000000.00`

## English

### Local Verification
- `cargo check --workspace` passed and verified the `0.1.1` build surface for `eml-rs`, `eml-core`, and `eml-lowering`.
- `cargo fmt --check` passed.
- `cargo test -q` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo bench --bench eval_bench --no-run` passed.
- Note: this local machine does not have the `x86_64-unknown-linux-gnu` Rust target installed, so the local `cargo check --workspace --target x86_64-unknown-linux-gnu` failure is an environment issue; remote CI installs and covers that target.

### Remote PR Verification
- PR: `https://github.com/fangfuzha/eml-rs/pull/3`
- Merge commit: `12d1413`
- PR CI runs: `24891283888`, `24891285458`
- Result: `changes`, `test`, `lint`, `no_std`, `coverage`, `supply-chain`, and `compat` all passed.
- Follow-up main CI run: `24892142360`, verifying that the `Node.js 20` runtime annotation disappeared after switching to `dorny/paths-filter@v4.0.1`.

### Manual Nightly Bench-Only
- Run: `https://github.com/fangfuzha/eml-rs/actions/runs/24891397303`
- Job: `nightly-bench` (`72884405360`)
- Result: passed in about `3m58s`.
- benchmark gate:
  - `rpn_vs_tree_median = 1.0230`, limit `1.1500`
  - `bytecode_vs_tree_median = 0.1693`, limit `0.8500`
  - `bytecode_vs_tree_p99 = 0.1697`, limit `0.9500`
  - `softmax_ce_bytecode_vs_tree_median_batch1024 = 0.7369`, limit `0.8500`
  - `softmax_ce_bytecode_vs_tree_p99_batch1024 = 0.7417`, limit `0.9500`
  - `lower_verify_10k_nodes_p99 = 1111634.45 ns/iter`, limit `25000000.00`
