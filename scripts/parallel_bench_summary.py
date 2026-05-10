#!/usr/bin/env python3
"""Summarize Criterion results for the experimental parallel benchmark suite."""

from __future__ import annotations

import argparse
import json
import pathlib
from datetime import datetime, timezone
from typing import Any, Dict, List


def load_estimate(path: pathlib.Path) -> Dict[str, float]:
    data = json.loads(path.read_text(encoding="utf-8"))
    out: Dict[str, float] = {}
    for name in ("mean", "median", "std_dev"):
        if name in data:
            out[name] = float(data[name]["point_estimate"])
    return out


def collect_estimates(criterion_dir: pathlib.Path) -> Dict[str, Dict[str, float]]:
    benchmarks: Dict[str, Dict[str, float]] = {}
    for bench_dir in criterion_dir.iterdir():
        if not bench_dir.is_dir():
            continue
        estimate_path = bench_dir / "new" / "estimates.json"
        if not estimate_path.exists():
            continue
        benchmarks[bench_dir.name] = load_estimate(estimate_path)
    return benchmarks


def ratio(lhs: float, rhs: float) -> float | None:
    if rhs == 0.0:
        return None
    return lhs / rhs


def collect_bytecode_modes(
    benchmarks: Dict[str, Dict[str, float]],
) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    for batch_size in (256, 1024, 4096):
        off = benchmarks[f"parallel_bytecode_off_batch{batch_size}"]
        auto = benchmarks[f"parallel_bytecode_auto_batch{batch_size}"]
        force = benchmarks[f"parallel_bytecode_force_batch{batch_size}"]
        out.append(
            {
                "batch_size": batch_size,
                "off": off,
                "auto": auto,
                "force": force,
                "ratios": {
                    "auto_vs_off_median": ratio(auto["median"], off["median"]),
                    "force_vs_off_median": ratio(force["median"], off["median"]),
                    "force_vs_auto_median": ratio(force["median"], auto["median"]),
                },
            }
        )
    return out


def collect_parallel_thresholds(
    benchmarks: Dict[str, Dict[str, float]],
) -> Dict[str, List[Dict[str, Any]]]:
    out: Dict[str, List[Dict[str, Any]]] = {"tree": [], "rpn": []}
    for backend in out:
        for batch_size in (32, 64, 128, 256, 512, 1024):
            serial = benchmarks[f"parallel_{backend}_serial_batch{batch_size}"]
            auto = benchmarks[f"parallel_{backend}_auto_batch{batch_size}"]
            out[backend].append(
                {
                    "batch_size": batch_size,
                    "serial": serial,
                    "auto": auto,
                    "ratio_auto_vs_serial_median": ratio(auto["median"], serial["median"]),
                }
            )
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--criterion-dir", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    criterion_dir = pathlib.Path(args.criterion_dir)
    output_path = pathlib.Path(args.output)
    if not criterion_dir.exists():
        raise SystemExit(f"missing criterion directory: {criterion_dir}")

    benchmarks = collect_estimates(criterion_dir)
    summary = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "criterion_dir": str(criterion_dir),
        "bytecode_modes": collect_bytecode_modes(benchmarks),
        "parallel_thresholds": collect_parallel_thresholds(benchmarks),
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())