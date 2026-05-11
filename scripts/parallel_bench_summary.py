#!/usr/bin/env python3
"""Summarize Criterion results for the experimental parallel benchmark suite."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List

BYTECODE_AUTO_MAX_WORKERS = 8
BYTECODE_AUTO_MIN_SAMPLES_PER_WORKER = 256


def load_estimate(path: pathlib.Path) -> Dict[str, float]:
  data = json.loads(path.read_text(encoding="utf-8"))
  out: Dict[str, float] = {}
  for name in ("mean", "median", "std_dev"):
    if name in data:
      out[name] = float(data[name]["point_estimate"])
  return out


def collect_estimates(
    criterion_dir: pathlib.Path) -> Dict[str, Dict[str, float]]:
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


def median_winner(modes: Dict[str, Dict[str, float]]) -> str:
  """Return the mode with the lowest median point estimate."""
  return min(modes, key=lambda name: modes[name]["median"])


def ranked_modes(modes: Dict[str, Dict[str, float]]) -> List[tuple[str, float]]:
  """Return modes sorted by median point estimate ascending."""
  return sorted(
      ((name, values["median"]) for name, values in modes.items()),
      key=lambda item: item[1],
  )


def recommendation_confidence(best: float, second_best: float) -> str:
  """Classify how strong the winner margin is versus the second-best mode."""
  if second_best <= 0.0:
    return "unknown"
  ratio_vs_second = best / second_best
  if ratio_vs_second <= 0.90:
    return "high"
  if ratio_vs_second <= 0.97:
    return "medium"
  return "low"


def collect_bytecode_modes(
    benchmarks: Dict[str, Dict[str, float]], ) -> List[Dict[str, Any]]:
  out: List[Dict[str, Any]] = []
  for batch_size in (256, 1024, 4096):
    off = benchmarks[f"parallel_bytecode_off_batch{batch_size}"]
    auto = benchmarks[f"parallel_bytecode_auto_batch{batch_size}"]
    force = benchmarks[f"parallel_bytecode_force_batch{batch_size}"]
    modes = {"off": off, "auto": auto, "force": force}
    ranking = ranked_modes(modes)
    out.append({
        "batch_size": batch_size,
        "off": off,
        "auto": auto,
        "force": force,
        "median_winner": median_winner(modes),
        "median_ranking": [{
          "mode": name,
          "median": median,
        } for name, median in ranking],
        "ratios": {
            "auto_vs_off_median": ratio(auto["median"], off["median"]),
            "force_vs_off_median": ratio(force["median"], off["median"]),
            "force_vs_auto_median": ratio(force["median"], auto["median"]),
        },
    })
  return out


def collect_bytecode_policy_analysis(
    bytecode_modes: List[Dict[str, Any]], ) -> Dict[str, Any]:
  """Summarize the configured default policy and batch-level winners."""
  batch_recommendations: List[Dict[str, Any]] = []
  for entry in bytecode_modes:
    ranking = entry["median_ranking"]
    best = ranking[0]
    second_best = ranking[1]
    batch_recommendations.append({
        "batch_size": entry["batch_size"],
        "recommended_mode": best["mode"],
        "confidence": recommendation_confidence(best["median"],
                                                  second_best["median"]),
        "winner_margin_vs_second": ratio(best["median"],
                                           second_best["median"]),
    })

  auto_vs_off_small = bytecode_modes[0]["ratios"]["auto_vs_off_median"]
  auto_vs_off_mid = bytecode_modes[1]["ratios"]["auto_vs_off_median"]
  auto_vs_off_large = bytecode_modes[2]["ratios"]["auto_vs_off_median"]
  keep_auto_default = (
      auto_vs_off_small is not None and auto_vs_off_small <= 1.10 and
      auto_vs_off_mid is not None and auto_vs_off_mid < 1.0 and
      auto_vs_off_large is not None and auto_vs_off_large < 1.0)
  default_reason = (
      "keep-auto-default" if keep_auto_default else
      "revisit-bytecode-auto-threshold")

  return {
    "configured_default": {
      "mode": "auto",
      "workers_cap": BYTECODE_AUTO_MAX_WORKERS,
      "min_samples_per_worker": BYTECODE_AUTO_MIN_SAMPLES_PER_WORKER,
    },
    "batch_median_winners": [{
      "batch_size": entry["batch_size"],
      "winner": entry["median_winner"],
    } for entry in bytecode_modes],
    "batch_recommendations": batch_recommendations,
    "default_policy_recommendation": {
      "recommended_mode":
      "auto" if keep_auto_default else batch_recommendations[-1]["recommended_mode"],
      "keep_configured_default": keep_auto_default,
      "reason": default_reason,
      "evidence": {
        "auto_vs_off_batch256": auto_vs_off_small,
        "auto_vs_off_batch1024": auto_vs_off_mid,
        "auto_vs_off_batch4096": auto_vs_off_large,
      },
    },
  }


def collect_parallel_thresholds(
    benchmarks: Dict[str, Dict[str,
                               float]], ) -> Dict[str, List[Dict[str, Any]]]:
  out: Dict[str, List[Dict[str, Any]]] = {"tree": [], "rpn": []}
  for backend in out:
    for batch_size in (32, 64, 128, 256, 512, 1024):
      serial = benchmarks[f"parallel_{backend}_serial_batch{batch_size}"]
      auto = benchmarks[f"parallel_{backend}_auto_batch{batch_size}"]
      out[backend].append({
          "batch_size":
          batch_size,
          "serial":
          serial,
          "auto":
          auto,
          "ratio_auto_vs_serial_median":
          ratio(auto["median"], serial["median"]),
      })
  return out
def format_policy_recommendation(summary: Dict[str, Any]) -> List[str]:
  """Render a concise human-readable recommendation block."""
  analysis = summary["bytecode_policy_analysis"]
  configured = analysis["configured_default"]
  default_recommendation = analysis["default_policy_recommendation"]

  lines = [
      "[parallel-summary] configured_default="
      f"{configured['mode']} workers_cap={configured['workers_cap']} "
      f"min_samples_per_worker={configured['min_samples_per_worker']}"
  ]
  for entry in analysis["batch_recommendations"]:
    margin = entry["winner_margin_vs_second"]
    lines.append(
        "[parallel-summary] batch="
        f"{entry['batch_size']} recommended_mode={entry['recommended_mode']} "
        f"confidence={entry['confidence']} margin_vs_second={margin:.4f}")
  lines.append(
      "[parallel-summary] default_policy_recommendation="
      f"{default_recommendation['reason']} keep={default_recommendation['keep_configured_default']} "
      f"recommended_mode={default_recommendation['recommended_mode']}")
  return lines


def main() -> int:
  parser = argparse.ArgumentParser()
  parser.add_argument("--criterion-dir", required=True)
  parser.add_argument("--output", required=True)
  parser.add_argument("--print-recommendation", action="store_true")
  parser.add_argument("--require-keep-configured-default", action="store_true")
  parser.add_argument(
      "--require-default-mode",
      choices=("off", "auto", "force"),
  )
  args = parser.parse_args()

  criterion_dir = pathlib.Path(args.criterion_dir)
  output_path = pathlib.Path(args.output)
  if not criterion_dir.exists():
    raise SystemExit(f"missing criterion directory: {criterion_dir}")

  benchmarks = collect_estimates(criterion_dir)
  bytecode_modes = collect_bytecode_modes(benchmarks)
  summary = {
      "schema_version": 1,
      "generated_at": datetime.now(timezone.utc).isoformat(),
      "criterion_dir": str(criterion_dir),
      "bytecode_modes": bytecode_modes,
      "bytecode_policy_analysis":
      collect_bytecode_policy_analysis(bytecode_modes),
      "parallel_thresholds": collect_parallel_thresholds(benchmarks),
  }

  output_path.parent.mkdir(parents=True, exist_ok=True)
  output_path.write_text(json.dumps(summary, indent=2, sort_keys=True),
                         encoding="utf-8")
  print(json.dumps(summary, indent=2, sort_keys=True))

  if args.print_recommendation:
    for line in format_policy_recommendation(summary):
      print(line)

  recommended = summary["bytecode_policy_analysis"]["default_policy_recommendation"]
  if args.require_keep_configured_default and not recommended[
      "keep_configured_default"]:
    print(
        "configured default policy is no longer recommended",
        file=sys.stderr,
    )
    return 1
  if args.require_default_mode and recommended[
      "recommended_mode"] != args.require_default_mode:
    print(
        "recommended default mode mismatch: "
        f"expected {args.require_default_mode}, got {recommended['recommended_mode']}",
        file=sys.stderr,
    )
    return 1

  return 0


if __name__ == "__main__":
  raise SystemExit(main())
