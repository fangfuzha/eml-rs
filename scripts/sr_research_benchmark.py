#!/usr/bin/env python3
"""Run a deterministic symbolic-regression research benchmark summary."""

from __future__ import annotations

import argparse
import json
import math
import pathlib
import random
import sys
import time
from datetime import datetime, timezone
from typing import Any, Callable, Dict, List

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_PARAMS = {"w0": 1.3, "w1": -0.2, "w2": -0.7, "w3": 0.4}
INITIALIZATION_SCALE = 0.45


def predict(params: Dict[str, float], x_value: float) -> float:
  """Evaluate the fixed EML research template for one scalar input."""
  lhs = params["w0"] * x_value + params["w1"]
  rhs_linear = params["w2"] * x_value + params["w3"]
  try:
    rhs = math.exp(rhs_linear)
    value = math.exp(lhs) - math.log(rhs)
  except (OverflowError, ValueError):
    return math.nan
  if not math.isfinite(value):
    return math.nan
  return max(min(value, 1.0e6), -1.0e6)


def make_dataset(sample_count: int) -> List[Dict[str, float]]:
  """Build a deterministic one-dimensional regression dataset."""
  if sample_count < 2:
    raise SystemExit("--samples must be at least 2")
  out: List[Dict[str, float]] = []
  for index in range(sample_count):
    ratio = index / (sample_count - 1)
    x_value = -2.0 + 4.0 * ratio
    out.append({"x": x_value, "y": predict(TARGET_PARAMS, x_value)})
  return out


def initial_params(depth: int, seed: int) -> Dict[str, float]:
  """Create deterministic depth-dependent initialization."""
  rng = random.Random(seed + depth * 7919)
  return {
      key: rng.uniform(-INITIALIZATION_SCALE, INITIALIZATION_SCALE)
      for key in ("w0", "w1", "w2", "w3")
  }


def loss_and_incidence(params: Dict[str, float],
                       dataset: List[Dict[str, float]]) -> tuple[float, int]:
  """Return mean squared error and NaN/overflow incidence."""
  total = 0.0
  bad = 0
  for sample in dataset:
    y_pred = predict(params, sample["x"])
    if not math.isfinite(y_pred):
      bad += 1
      total += 1.0e12
      continue
    err = y_pred - sample["y"]
    total += err * err
  return total / len(dataset), bad


def finite_diff_grad(params: Dict[str, float], dataset: List[Dict[str, float]],
                     eps: float) -> Dict[str, float]:
  """Estimate a finite-difference gradient for the fixed template."""
  base, _ = loss_and_incidence(params, dataset)
  grad: Dict[str, float] = {}
  for key in params:
    shifted = dict(params)
    shifted[key] += eps
    shifted_loss, _ = loss_and_incidence(shifted, dataset)
    grad[key] = (shifted_loss - base) / eps
  return grad


def train_depth(depth: int, dataset: List[Dict[str, float]], seed: int,
                steps: int) -> Dict[str, Any]:
  """Run one deterministic SR recovery attempt for a depth bucket."""
  started = time.perf_counter()
  params = initial_params(depth, seed)
  lr = 0.0007 / math.sqrt(depth)
  eps = 1.0e-5
  nan_overflow_incidence = 0
  initial_loss, initial_bad = loss_and_incidence(params, dataset)
  nan_overflow_incidence += initial_bad

  for _ in range(steps):
    grad = finite_diff_grad(params, dataset, eps)
    for key, value in grad.items():
      clipped = max(min(value, 50.0), -50.0)
      params[key] -= lr * clipped
    _, bad = loss_and_incidence(params, dataset)
    nan_overflow_incidence += bad

  final_loss, final_bad = loss_and_incidence(params, dataset)
  nan_overflow_incidence += final_bad
  param_error = math.sqrt(
      sum((params[key] - TARGET_PARAMS[key])**2
          for key in params) / len(params))
  recovered = final_loss <= 1.0e-3
  snapped = param_error <= 0.20
  elapsed = time.perf_counter() - started
  return {
      "depth": depth,
      "samples": len(dataset),
      "initialization_strategy": "deterministic-uniform-depth-seeded",
      "hardening": {
          "gradient_clip": 50.0,
          "output_clamp_abs": 1000000.0,
          "finite_difference_eps": eps,
      },
      "steps": steps,
      "initial_loss": initial_loss,
      "final_loss": final_loss,
      "param_rmse": param_error,
      "recovered": recovered,
      "snapped_to_symbolic": snapped,
      "nan_overflow_incidence": nan_overflow_incidence,
      "wall_time_ms": elapsed * 1000.0,
      "learned_params": params,
  }


def aggregate(results: List[Dict[str, Any]]) -> Dict[str, Any]:
  """Aggregate per-depth benchmark results."""
  total = len(results)
  recovered = sum(1 for item in results if item["recovered"])
  snapped = sum(1 for item in results if item["snapped_to_symbolic"])
  return {
      "depth_min":
      min(item["depth"] for item in results),
      "depth_max":
      max(item["depth"] for item in results),
      "runs":
      total,
      "recovery_rate":
      recovered / total,
      "snap_to_symbolic_rate":
      snapped / total,
      "nan_overflow_incidence":
      sum(item["nan_overflow_incidence"] for item in results),
      "wall_time_ms":
      sum(item["wall_time_ms"] for item in results),
  }


def build_summary(sample_count: int, seed: int, steps: int) -> Dict[str, Any]:
  """Build a full SR research summary for depths 2 through 6."""
  dataset = make_dataset(sample_count)
  results = [train_depth(depth, dataset, seed, steps) for depth in range(2, 7)]
  return {
      "schema": "eml-rs.sr-research-benchmark.v1",
      "generated_at": datetime.now(timezone.utc).isoformat(),
      "platform_scope": "linux-primary-non-blocking-nightly",
      "template": "eml(w0*x + w1, exp(w2*x + w3))",
      "target_params": TARGET_PARAMS,
      "sample_count": sample_count,
      "seed": seed,
      "depths": [2, 3, 4, 5, 6],
      "metrics": aggregate(results),
      "runs": results,
      "governance": {
          "track": "symbolic-regression-research",
          "blocking_gate": False,
          "primary_validation_platform": "linux",
      },
  }


def render_markdown(summary: Dict[str, Any]) -> str:
  """Render a human-readable SR research summary."""
  metrics = summary["metrics"]
  lines = [
      "# Symbolic Regression Research Benchmark",
      "",
      f"- Schema: `{summary['schema']}`",
      f"- Generated at: `{summary['generated_at']}`",
      f"- Template: `{summary['template']}`",
      f"- Depths: `{summary['depths']}`",
      f"- Sample count: `{summary['sample_count']}`",
      f"- Recovery rate: `{metrics['recovery_rate']:.3f}`",
      f"- Snap-to-symbolic rate: `{metrics['snap_to_symbolic_rate']:.3f}`",
      f"- NaN/overflow incidence: `{metrics['nan_overflow_incidence']}`",
      f"- Wall time ms: `{metrics['wall_time_ms']:.3f}`",
      f"- Blocking gate: `{summary['governance']['blocking_gate']}`",
      "",
      "## Per-depth Runs",
      "",
      "| Depth | Final loss | Param RMSE | Recovered | Snapped | NaN/overflow | Wall ms |",
      "| ----- | ---------- | ---------- | --------- | ------- | ------------ | ------- |",
  ]
  for run in summary["runs"]:
    lines.append("| `{depth}` | `{loss:.6e}` | `{rmse:.6e}` | `{recovered}` | "
                 "`{snapped}` | `{bad}` | `{wall:.3f}` |".format(
                     depth=run["depth"],
                     loss=run["final_loss"],
                     rmse=run["param_rmse"],
                     recovered=run["recovered"],
                     snapped=run["snapped_to_symbolic"],
                     bad=run["nan_overflow_incidence"],
                     wall=run["wall_time_ms"],
                 ))
  lines.append("")
  return "\n".join(lines)


def resolve_path(path: str) -> pathlib.Path:
  """Resolve a user path relative to the repository root."""
  candidate = pathlib.Path(path)
  if candidate.is_absolute():
    return candidate
  return ROOT / candidate


def write_text(path: pathlib.Path, text_factory: Callable[[], str]) -> None:
  """Create parent directories and write UTF-8 text."""
  path.parent.mkdir(parents=True, exist_ok=True)
  path.write_text(text_factory(), encoding="utf-8")


def main() -> int:
  """Run the SR research benchmark CLI."""
  parser = argparse.ArgumentParser()
  parser.add_argument("--samples", type=int, default=41)
  parser.add_argument("--seed", type=int, default=1729)
  parser.add_argument("--steps", type=int, default=80)
  parser.add_argument("--output-json",
                      default="target/sr-research-benchmark.json")
  parser.add_argument("--output-md", default="target/sr-research-benchmark.md")
  args = parser.parse_args()

  if args.steps <= 0:
    raise SystemExit("--steps must be positive")

  summary = build_summary(args.samples, args.seed, args.steps)
  json_text = json.dumps(summary, indent=2, sort_keys=True)
  write_text(resolve_path(args.output_json), lambda: json_text + "\n")
  write_text(resolve_path(args.output_md), lambda: render_markdown(summary))
  print(json_text)
  print(
      "[sr-research] depths=2..6 recovery_rate="
      f"{summary['metrics']['recovery_rate']:.3f} snap_to_symbolic_rate="
      f"{summary['metrics']['snap_to_symbolic_rate']:.3f} nan_overflow_incidence="
      f"{summary['metrics']['nan_overflow_incidence']}")
  return 0


if __name__ == "__main__":
  sys.exit(main())
