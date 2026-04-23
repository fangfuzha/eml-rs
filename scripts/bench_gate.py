#!/usr/bin/env python3
"""Simple Criterion-based benchmark regression gate.

The script loads `new/estimates.json` for each benchmark and verifies
ratio constraints from a JSON config.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from typing import Dict


def read_point_estimate(path: pathlib.Path) -> float:
    data = json.loads(path.read_text(encoding="utf-8"))
    return float(data["mean"]["point_estimate"])


def collect_means(criterion_dir: pathlib.Path) -> Dict[str, float]:
    out: Dict[str, float] = {}
    for bench_dir in criterion_dir.iterdir():
        if not bench_dir.is_dir():
            continue
        est = bench_dir / "new" / "estimates.json"
        if est.exists():
            out[bench_dir.name] = read_point_estimate(est)
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--criterion-dir", required=True)
    parser.add_argument("--config", required=True)
    args = parser.parse_args()

    criterion_dir = pathlib.Path(args.criterion_dir)
    config_path = pathlib.Path(args.config)
    if not criterion_dir.exists():
        print(f"[bench-gate] missing criterion directory: {criterion_dir}")
        return 2
    if not config_path.exists():
        print(f"[bench-gate] missing config: {config_path}")
        return 2

    config = json.loads(config_path.read_text(encoding="utf-8"))
    ratios = config.get("ratios", [])
    means = collect_means(criterion_dir)

    ok = True
    for rule in ratios:
        name = rule["name"]
        lhs = rule["lhs"]
        rhs = rule["rhs"]
        max_ratio = float(rule["max"])
        if lhs not in means or rhs not in means:
            print(f"[bench-gate] missing benchmark for rule '{name}': lhs={lhs} rhs={rhs}")
            ok = False
            continue
        ratio = means[lhs] / means[rhs]
        print(
            f"[bench-gate] {name}: {lhs}/{rhs} = {ratio:.4f} (max={max_ratio:.4f})"
        )
        if ratio > max_ratio:
            print(f"[bench-gate] FAIL rule '{name}'")
            ok = False

    if not ratios:
        print("[bench-gate] no ratio rules configured")
        return 2

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
