#!/usr/bin/env python3
"""Criterion-based benchmark regression gate.

The script reads `new/estimates.json` and `new/sample.json` from Criterion
output directories, then verifies:
- required benchmark presence
- ratio constraints for a chosen metric (`mean`, `median`, `p95`, `p99`)
- absolute upper bounds for a chosen metric
"""

from __future__ import annotations

import argparse
import json
import math
import pathlib
import sys
from typing import Dict, List


def read_point_estimate(path: pathlib.Path) -> Dict[str, float]:
    data = json.loads(path.read_text(encoding="utf-8"))
    metrics: Dict[str, float] = {}
    for name in ("mean", "median", "std_dev"):
        if name in data:
            metrics[name] = float(data[name]["point_estimate"])
    return metrics


def percentile(sorted_values: List[float], q: float) -> float:
    if not sorted_values:
        raise ValueError("cannot compute percentile of empty sample")
    if len(sorted_values) == 1:
        return sorted_values[0]

    rank = q * (len(sorted_values) - 1)
    lo = math.floor(rank)
    hi = math.ceil(rank)
    if lo == hi:
        return sorted_values[lo]
    frac = rank - lo
    return sorted_values[lo] * (1.0 - frac) + sorted_values[hi] * frac


def read_tukey_bounds(path: pathlib.Path) -> tuple[float, float] | None:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list) or len(data) != 4:
        return None
    low_mild = float(data[1])
    high_mild = float(data[2])
    return (low_mild, high_mild)


def filter_outliers(values: List[float], bounds: tuple[float, float] | None) -> List[float]:
    if bounds is None:
        return values
    low_mild, high_mild = bounds
    filtered = [value for value in values if low_mild <= value <= high_mild]
    return filtered or values


def read_sample_metrics(path: pathlib.Path, tukey_path: pathlib.Path) -> Dict[str, float]:
    data = json.loads(path.read_text(encoding="utf-8"))
    iters = data.get("iters", [])
    times = data.get("times", [])
    if not iters or len(iters) != len(times):
        raise ValueError(f"invalid sample file: {path}")

    per_iter_times: List[float] = []
    for iter_count, total_time in zip(iters, times):
        iter_count = float(iter_count)
        total_time = float(total_time)
        if iter_count <= 0.0:
            raise ValueError(f"non-positive iteration count in {path}")
        per_iter_times.append(total_time / iter_count)

    per_iter_times = filter_outliers(
        per_iter_times,
        read_tukey_bounds(tukey_path) if tukey_path.exists() else None,
    )
    per_iter_times.sort()
    return {
        "p95": percentile(per_iter_times, 0.95),
        "p99": percentile(per_iter_times, 0.99),
    }


def collect_metrics(criterion_dir: pathlib.Path) -> Dict[str, Dict[str, float]]:
    out: Dict[str, Dict[str, float]] = {}
    for bench_dir in criterion_dir.iterdir():
        if not bench_dir.is_dir():
            continue

        est = bench_dir / "new" / "estimates.json"
        sample = bench_dir / "new" / "sample.json"
        tukey = bench_dir / "new" / "tukey.json"
        if not est.exists():
            continue

        metrics = read_point_estimate(est)
        if sample.exists():
            metrics.update(read_sample_metrics(sample, tukey))
        out[bench_dir.name] = metrics
    return out


def read_metric(
    benchmarks: Dict[str, Dict[str, float]],
    benchmark_name: str,
    metric_name: str,
) -> float:
    metrics = benchmarks.get(benchmark_name)
    if metrics is None:
        raise KeyError(f"missing benchmark '{benchmark_name}'")
    if metric_name not in metrics:
        raise KeyError(
            f"missing metric '{metric_name}' for benchmark '{benchmark_name}'"
        )
    return metrics[metric_name]


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
    max_metrics = config.get("max_metrics", [])
    required = config.get("required", [])
    benchmarks = collect_metrics(criterion_dir)

    ok = True
    for bench_name in required:
        if bench_name not in benchmarks:
            print(f"[bench-gate] missing required benchmark: {bench_name}")
            ok = False

    for rule in ratios:
        name = rule["name"]
        lhs = rule["lhs"]
        rhs = rule["rhs"]
        metric = rule.get("metric", "mean")
        max_ratio = float(rule["max"])
        try:
            lhs_value = read_metric(benchmarks, lhs, metric)
            rhs_value = read_metric(benchmarks, rhs, metric)
        except KeyError as exc:
            print(f"[bench-gate] missing benchmark for rule '{name}': {exc}")
            ok = False
            continue
        ratio = lhs_value / rhs_value
        print(
            f"[bench-gate] {name}: {lhs}/{rhs} [{metric}] = {ratio:.4f} "
            f"(max={max_ratio:.4f})"
        )
        if ratio > max_ratio:
            print(f"[bench-gate] FAIL rule '{name}'")
            ok = False

    for rule in max_metrics:
        name = rule["name"]
        benchmark_name = rule["benchmark"]
        metric = rule.get("metric", "mean")
        max_value = float(rule["max"])
        try:
            value = read_metric(benchmarks, benchmark_name, metric)
        except KeyError as exc:
            print(f"[bench-gate] missing benchmark for rule '{name}': {exc}")
            ok = False
            continue
        print(
            f"[bench-gate] {name}: {benchmark_name} [{metric}] = {value:.2f} ns/iter "
            f"(max={max_value:.2f})"
        )
        if value > max_value:
            print(f"[bench-gate] FAIL rule '{name}'")
            ok = False

    if not ratios and not max_metrics and not required:
        print("[bench-gate] no rules configured")
        return 2

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
