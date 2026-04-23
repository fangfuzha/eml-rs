# v0.1.0 Verification Record / 发布前验证记录

## 中文

### 计划执行项
- `cargo fmt`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test -q`
- `cargo bench --no-run`
- `cargo bench --bench eval_bench`
- `python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json`
- `cargo build --release -q`
- `cargo audit`
- `cargo deny check licenses bans`

### 结果
- 执行时间：`2026-04-24T05:54:57.9786601+08:00`
- 本地结果：
  - `cargo fmt -- --check` 通过
  - `cargo check --all-targets` 通过
  - `cargo clippy --all-targets -- -D warnings` 通过
  - `cargo test -q` 通过
  - `cargo bench --no-run` 通过
  - `cargo bench --bench eval_bench` 通过
  - `python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json` 通过
  - `cargo build --release -q` 通过
  - `cargo audit` 通过，未发现 RustSec 漏洞
  - `cargo deny check licenses bans` 通过
- 额外说明：
  - 本机为补齐 supply-chain 检查，安装了 `cargo-audit` 与 `cargo-deny`
  - benchmark gate 关键门槛通过：`rpn/tree median=1.0554`、`shared bytecode/tree p99=0.2209`、`softmax_ce bytecode/tree p99=0.6306`、`lower_verify_10k_nodes p99=24.46ms`

## English

### Planned Checks
- `cargo fmt`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test -q`
- `cargo bench --no-run`
- `cargo bench --bench eval_bench`
- `python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json`
- `cargo build --release -q`
- `cargo audit`
- `cargo deny check licenses bans`

### Result
- Execution time: `2026-04-24T05:54:57.9786601+08:00`
- Local result:
  - `cargo fmt -- --check` passed
  - `cargo check --all-targets` passed
  - `cargo clippy --all-targets -- -D warnings` passed
  - `cargo test -q` passed
  - `cargo bench --no-run` passed
  - `cargo bench --bench eval_bench` passed
  - `python scripts/bench_gate.py --criterion-dir target/criterion --config benchmarks/gate.json` passed
  - `cargo build --release -q` passed
  - `cargo audit` passed with no RustSec vulnerabilities found
  - `cargo deny check licenses bans` passed
- Additional notes:
  - `cargo-audit` and `cargo-deny` were installed locally as part of release preparation
  - The key benchmark gates passed: `rpn/tree median=1.0554`, `shared bytecode/tree p99=0.2209`, `softmax_ce bytecode/tree p99=0.6306`, `lower_verify_10k_nodes p99=24.46ms`
