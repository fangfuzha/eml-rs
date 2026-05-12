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
from typing import Any, Callable, Dict, List, Sequence

ROOT = pathlib.Path(__file__).resolve().parents[1]
INITIALIZATION_SCALE = 0.45
OUTPUT_CLAMP_ABS = 1.0e6
RECOVERY_LOSS_THRESHOLD = 1.0e-3
SNAP_PARAM_RMSE_THRESHOLD = 0.20
NUMERICAL_EQUIVALENCE_MAX_ABS_ERROR = 1.0e-3
DEPTHS = [2, 3, 4, 5, 6]
DEFAULT_SEEDS = [1729, 2718, 3141]
PARAMETER_KEYS = ("w0", "w1", "w2", "w3")
SNAPPING_STATE_ORDER = {
    "symbolic-equivalent": 0,
    "numerically-equivalent-indeterminate": 1,
    "parameter-close-only": 2,
    "not-equivalent": 3,
    "unstable": 4,
}
TASK_SPECS: Dict[str, Dict[str, Any]] = {
    "exp-log": {
        "template": "eml(w0*x + w1, exp(w2*x + w3))",
        "family": "exp-log",
        "learning_rate": 0.0007,
        "target_params": {
            "w0": 1.3,
            "w1": -0.2,
            "w2": -0.7,
            "w3": 0.4,
        },
    },
    "trigonometric": {
        "template": "sin(w0*x + w1) + cos(w2*x + w3)",
        "family": "trigonometric",
        "learning_rate": 0.0025,
        "target_params": {
            "w0": 0.8,
            "w1": -0.4,
            "w2": -1.1,
            "w3": 0.6,
        },
    },
    "low-order-polynomial": {
        "template": "((w0*x + w1)*x + w2)*x + w3",
        "family": "low-order-polynomial",
        "learning_rate": 0.0015,
        "target_params": {
            "w0": 0.35,
            "w1": -0.8,
            "w2": 1.1,
            "w3": -0.2,
        },
    },
}


def clamp_output(value: float) -> float:
  """Clamp finite outputs into the benchmark's bounded numeric range."""
  if not math.isfinite(value):
    return math.nan
  return max(min(value, OUTPUT_CLAMP_ABS), -OUTPUT_CLAMP_ABS)


def task_names() -> List[str]:
  """Return the ordered symbolic-regression task set."""
  return list(TASK_SPECS.keys())


def task_spec(task_name: str) -> Dict[str, Any]:
  """Return the static configuration for a named SR task."""
  if task_name not in TASK_SPECS:
    raise SystemExit(f"unsupported task: {task_name}")
  return TASK_SPECS[task_name]


def predict_exp_log(params: Dict[str, float], x_value: float) -> float:
  """Evaluate the exp-log research template for one scalar input."""
  lhs = params["w0"] * x_value + params["w1"]
  rhs_linear = params["w2"] * x_value + params["w3"]
  try:
    rhs = math.exp(rhs_linear)
    value = math.exp(lhs) - math.log(rhs)
  except (OverflowError, ValueError):
    return math.nan
  return clamp_output(value)


def predict_trigonometric(params: Dict[str, float], x_value: float) -> float:
  """Evaluate the trigonometric research template for one scalar input."""
  value = math.sin(params["w0"] * x_value + params["w1"])
  value += math.cos(params["w2"] * x_value + params["w3"])
  return clamp_output(value)


def predict_polynomial(params: Dict[str, float], x_value: float) -> float:
  """Evaluate the low-order polynomial research template for one scalar input."""
  value = ((params["w0"] * x_value + params["w1"]) * x_value +
           params["w2"]) * x_value + params["w3"]
  return clamp_output(value)


def predict(task_name: str, params: Dict[str, float], x_value: float) -> float:
  """Dispatch one scalar prediction for the configured research task."""
  if task_name == "exp-log":
    return predict_exp_log(params, x_value)
  if task_name == "trigonometric":
    return predict_trigonometric(params, x_value)
  if task_name == "low-order-polynomial":
    return predict_polynomial(params, x_value)
  raise SystemExit(f"unsupported task: {task_name}")


def make_dataset(task_name: str, sample_count: int) -> List[Dict[str, float]]:
  """Build a deterministic one-dimensional dataset for a named SR task."""
  if sample_count < 2:
    raise SystemExit("--samples must be at least 2")
  target_params = task_spec(task_name)["target_params"]
  out: List[Dict[str, float]] = []
  for index in range(sample_count):
    ratio = index / (sample_count - 1)
    x_value = -2.0 + 4.0 * ratio
    out.append({"x": x_value, "y": predict(task_name, target_params, x_value)})
  return out


def initial_params(task_name: str, depth: int, seed: int) -> Dict[str, float]:
  """Create deterministic task- and depth-dependent initialization."""
  task_offset = task_names().index(task_name) * 104729
  rng = random.Random(seed + depth * 7919 + task_offset)
  return {
      key: rng.uniform(-INITIALIZATION_SCALE, INITIALIZATION_SCALE)
      for key in PARAMETER_KEYS
  }


def loss_and_incidence(task_name: str, params: Dict[str, float],
                       dataset: List[Dict[str, float]]) -> tuple[float, int]:
  """Return mean squared error and NaN/overflow incidence for one task."""
  total = 0.0
  bad = 0
  for sample in dataset:
    y_pred = predict(task_name, params, sample["x"])
    if not math.isfinite(y_pred):
      bad += 1
      total += 1.0e12
      continue
    err = y_pred - sample["y"]
    total += err * err
  return total / len(dataset), bad


def max_abs_error(task_name: str, params: Dict[str, float],
                  dataset: List[Dict[str, float]]) -> float:
  """Return the maximum absolute prediction error over the sampled domain."""
  maximum = 0.0
  for sample in dataset:
    y_pred = predict(task_name, params, sample["x"])
    if not math.isfinite(y_pred):
      return math.inf
    maximum = max(maximum, abs(y_pred - sample["y"]))
  return maximum


def finite_diff_grad(task_name: str, params: Dict[str, float],
                     dataset: List[Dict[str, float]],
                     eps: float) -> Dict[str, float]:
  """Estimate a finite-difference gradient for a fixed task template."""
  base, _ = loss_and_incidence(task_name, params, dataset)
  grad: Dict[str, float] = {}
  for key in params:
    shifted = dict(params)
    shifted[key] += eps
    shifted_loss, _ = loss_and_incidence(task_name, shifted, dataset)
    grad[key] = (shifted_loss - base) / eps
  return grad


def snapping_state_for_run(final_loss: float, param_error: float,
                           max_error: float, final_bad_samples: int) -> str:
  """Classify one run using the repository's snapping governance rules."""
  if final_bad_samples > 0:
    return "unstable"
  numerically_equivalent = (final_loss <= RECOVERY_LOSS_THRESHOLD and max_error
                            <= NUMERICAL_EQUIVALENCE_MAX_ABS_ERROR)
  parameter_close = param_error <= SNAP_PARAM_RMSE_THRESHOLD
  if parameter_close and numerically_equivalent:
    return "symbolic-equivalent"
  if numerically_equivalent:
    return "numerically-equivalent-indeterminate"
  if parameter_close:
    return "parameter-close-only"
  return "not-equivalent"


def train_depth(task_name: str, depth: int, dataset: List[Dict[str, float]],
                seed: int, steps: int) -> Dict[str, Any]:
  """Run one deterministic SR recovery attempt for one task/depth pair."""
  started = time.perf_counter()
  spec = task_spec(task_name)
  params = initial_params(task_name, depth, seed)
  lr = spec["learning_rate"] / math.sqrt(depth)
  eps = 1.0e-5
  nan_overflow_incidence = 0
  initial_loss, initial_bad = loss_and_incidence(task_name, params, dataset)
  nan_overflow_incidence += initial_bad

  for _ in range(steps):
    grad = finite_diff_grad(task_name, params, dataset, eps)
    for key, value in grad.items():
      clipped = max(min(value, 50.0), -50.0)
      params[key] -= lr * clipped
    _, bad = loss_and_incidence(task_name, params, dataset)
    nan_overflow_incidence += bad

  final_loss, final_bad = loss_and_incidence(task_name, params, dataset)
  nan_overflow_incidence += final_bad
  max_error = max_abs_error(task_name, params, dataset)
  target_params = spec["target_params"]
  param_error = math.sqrt(
      sum((params[key] - target_params[key])**2
          for key in params) / len(params))
  recovered = final_loss <= RECOVERY_LOSS_THRESHOLD
  snapping_state = snapping_state_for_run(final_loss, param_error, max_error,
                                          final_bad)
  snapped = snapping_state == "symbolic-equivalent"
  elapsed = time.perf_counter() - started
  return {
      "task": task_name,
      "task_family": spec["family"],
      "template": spec["template"],
      "seed": seed,
      "depth": depth,
      "samples": len(dataset),
      "initialization_strategy": "deterministic-uniform-depth-seeded",
      "hardening": {
          "gradient_clip": 50.0,
          "output_clamp_abs": OUTPUT_CLAMP_ABS,
          "finite_difference_eps": eps,
      },
      "steps": steps,
      "initial_loss": initial_loss,
      "final_loss": final_loss,
      "param_rmse": param_error,
      "max_abs_error": max_error,
      "recovered": recovered,
      "snapped_to_symbolic": snapped,
      "snapping_state": snapping_state,
      "nan_overflow_incidence": nan_overflow_incidence,
      "final_bad_samples": final_bad,
      "wall_time_ms": elapsed * 1000.0,
      "learned_params": params,
  }


def mean_value(values: Sequence[float]) -> float:
  """Return the arithmetic mean of a numeric sequence."""
  if not values:
    return 0.0
  return sum(values) / len(values)


def variance_value(values: Sequence[float]) -> float:
  """Return the population variance of a numeric sequence."""
  if len(values) <= 1:
    return 0.0
  mean = mean_value(values)
  return sum((value - mean)**2 for value in values) / len(values)


def run_snapshot(run: Dict[str, Any]) -> Dict[str, Any]:
  """Return a compact run summary suitable for aggregate artifacts."""
  return {
      "task": run["task"],
      "seed": run["seed"],
      "depth": run["depth"],
      "final_loss": run["final_loss"],
      "param_rmse": run["param_rmse"],
      "max_abs_error": run["max_abs_error"],
      "snapping_state": run["snapping_state"],
      "nan_overflow_incidence": run["nan_overflow_incidence"],
      "wall_time_ms": run["wall_time_ms"],
  }


def best_run(results: List[Dict[str, Any]]) -> Dict[str, Any]:
  """Return the strongest run under the repository's SR ordering."""
  candidate = min(
      results,
      key=lambda item: (
          SNAPPING_STATE_ORDER[item["snapping_state"]],
          item["final_loss"],
          item["param_rmse"],
          item["wall_time_ms"],
      ),
  )
  return run_snapshot(candidate)


def worst_run(results: List[Dict[str, Any]]) -> Dict[str, Any]:
  """Return the weakest run under the repository's SR ordering."""
  candidate = max(
      results,
      key=lambda item: (
          SNAPPING_STATE_ORDER[item["snapping_state"]],
          item["final_loss"],
          item["param_rmse"],
          item["nan_overflow_incidence"],
          item["wall_time_ms"],
      ),
  )
  return run_snapshot(candidate)


def failure_summary(results: List[Dict[str, Any]]) -> Dict[str, Any]:
  """Summarize failed or indeterminate SR runs for artifact review."""
  failures = [
      item for item in results
      if not item["recovered"] or not item["snapped_to_symbolic"]
  ]
  by_reason = {
      "not_recovered":
      sum(1 for item in failures if not item["recovered"]),
      "not_symbolic_equivalent":
      sum(1 for item in failures if not item["snapped_to_symbolic"]),
      "unstable":
      sum(1 for item in failures if item["snapping_state"] == "unstable"),
      "numerically_equivalent_indeterminate":
      sum(1 for item in failures
          if item["snapping_state"] == "numerically-equivalent-indeterminate"),
      "parameter_close_only":
      sum(1 for item in failures
          if item["snapping_state"] == "parameter-close-only"),
      "not_equivalent":
      sum(1 for item in failures
          if item["snapping_state"] == "not-equivalent"),
  }
  ranked = sorted(
      failures,
      key=lambda item: (
          SNAPPING_STATE_ORDER[item["snapping_state"]],
          item["final_loss"],
          item["param_rmse"],
          item["nan_overflow_incidence"],
      ),
      reverse=True,
  )
  return {
      "failed_run_count": len(failures),
      "failure_rate": len(failures) / len(results) if results else 0.0,
      "by_reason": by_reason,
      "examples": [run_snapshot(item) for item in ranked[:5]],
  }


def aggregate(results: List[Dict[str, Any]]) -> Dict[str, Any]:
  """Aggregate per-task or cross-task symbolic-regression results."""
  if not results:
    raise SystemExit("expected at least one SR benchmark result")
  total = len(results)
  recovered = sum(1 for item in results if item["recovered"])
  snapped = sum(1 for item in results if item["snapped_to_symbolic"])
  state_counts = {
      state: sum(1 for item in results if item["snapping_state"] == state)
      for state in SNAPPING_STATE_ORDER
  }
  final_losses = [item["final_loss"] for item in results]
  param_rmses = [item["param_rmse"] for item in results]
  max_abs_errors = [item["max_abs_error"] for item in results]
  nan_incidence = [float(item["nan_overflow_incidence"]) for item in results]
  wall_times = [item["wall_time_ms"] for item in results]
  return {
      "depth_min": min(item["depth"] for item in results),
      "depth_max": max(item["depth"] for item in results),
      "runs": total,
      "seed_set": sorted({int(item["seed"])
                          for item in results}),
      "task_set": sorted({str(item["task"])
                          for item in results}),
      "recovery_rate": recovered / total,
      "snap_to_symbolic_rate": snapped / total,
      "snapping_state_counts": state_counts,
      "final_loss_mean": mean_value(final_losses),
      "final_loss_variance": variance_value(final_losses),
      "param_rmse_mean": mean_value(param_rmses),
      "param_rmse_variance": variance_value(param_rmses),
      "max_abs_error_mean": mean_value(max_abs_errors),
      "max_abs_error_variance": variance_value(max_abs_errors),
      "nan_overflow_incidence_mean": mean_value(nan_incidence),
      "nan_overflow_incidence_variance": variance_value(nan_incidence),
      "wall_time_ms_mean": mean_value(wall_times),
      "wall_time_ms_variance": variance_value(wall_times),
      "best_run": best_run(results),
      "worst_run": worst_run(results),
      "failure_summary": failure_summary(results),
  }


def build_summary(sample_count: int, seeds: List[int],
                  steps: int) -> Dict[str, Any]:
  """Build a full SR research summary for the configured task set."""
  task_metrics: List[Dict[str, Any]] = []
  all_runs: List[Dict[str, Any]] = []
  for task_name in task_names():
    spec = task_spec(task_name)
    dataset = make_dataset(task_name, sample_count)
    runs = [
        train_depth(task_name, depth, dataset, seed, steps) for seed in seeds
        for depth in DEPTHS
    ]
    all_runs.extend(runs)
    task_metrics.append({
        "name": task_name,
        "template": spec["template"],
        "family": spec["family"],
        "target_params": spec["target_params"],
        "metrics": aggregate(runs),
    })
  overall_metrics = aggregate(all_runs)
  return {
      "schema":
      "eml-rs.sr-research-benchmark.v2",
      "generated_at":
      datetime.now(timezone.utc).isoformat(),
      "platform_scope":
      "linux-primary-non-blocking-nightly",
      "tasks": [{
          "name": task_name,
          "template": task_spec(task_name)["template"],
          "family": task_spec(task_name)["family"],
          "target_params": task_spec(task_name)["target_params"],
      } for task_name in task_names()],
      "sample_count":
      sample_count,
      "seed_set":
      seeds,
      "depths":
      DEPTHS,
      "steps":
      steps,
      "snapping_rules": {
          "expression_equivalence":
          "fixed-template-family proxy via parameter RMSE; no algebraic canonicalization",
          "parameter_rmse_tolerance":
          SNAP_PARAM_RMSE_THRESHOLD,
          "numerical_equivalence": {
              "sample_domain": "x in [-2.0, 2.0] over the generated dataset",
              "max_abs_error_tolerance": NUMERICAL_EQUIVALENCE_MAX_ABS_ERROR,
              "final_loss_tolerance": RECOVERY_LOSS_THRESHOLD,
          },
          "indeterminate_state":
          "numerically equivalent but parameter RMSE exceeds tolerance, e.g. periodic trig aliases",
      },
      "metrics":
      overall_metrics,
      "task_metrics":
      task_metrics,
      "runs":
      all_runs,
      "governance": {
          "track": "symbolic-regression-research",
          "blocking_gate": False,
          "primary_validation_platform": "linux",
          "artifact_policy": "nightly-and-workflow-dispatch-non-blocking",
      },
  }


def format_run_reference(run: Dict[str, Any]) -> str:
  """Render a compact Markdown-friendly run reference."""
  return ("task={task} seed={seed} depth={depth} state={state} "
          "loss={loss:.6e} rmse={rmse:.6e} max_abs={max_abs:.6e} bad={bad} "
          "wall_ms={wall:.3f}").format(
              task=run["task"],
              seed=run["seed"],
              depth=run["depth"],
              state=run["snapping_state"],
              loss=run["final_loss"],
              rmse=run["param_rmse"],
              max_abs=run["max_abs_error"],
              bad=run["nan_overflow_incidence"],
              wall=run["wall_time_ms"],
          )


def render_markdown(summary: Dict[str, Any]) -> str:
  """Render a human-readable SR research summary."""
  metrics = summary["metrics"]
  lines = [
      "# Symbolic Regression Research Benchmark",
      "",
      f"- Schema: `{summary['schema']}`",
      f"- Generated at: `{summary['generated_at']}`",
      f"- Tasks: `{[task['name'] for task in summary['tasks']]}`",
      f"- Depths: `{summary['depths']}`",
      f"- Seed set: `{summary['seed_set']}`",
      f"- Sample count: `{summary['sample_count']}`",
      f"- Steps: `{summary['steps']}`",
      f"- Recovery rate: `{metrics['recovery_rate']:.3f}`",
      f"- Snap-to-symbolic rate: `{metrics['snap_to_symbolic_rate']:.3f}`",
      f"- Final loss mean/variance: `{metrics['final_loss_mean']:.6e}` / `{metrics['final_loss_variance']:.6e}`",
      f"- Param RMSE mean/variance: `{metrics['param_rmse_mean']:.6e}` / `{metrics['param_rmse_variance']:.6e}`",
      f"- NaN/overflow incidence mean/variance: `{metrics['nan_overflow_incidence_mean']:.3f}` / `{metrics['nan_overflow_incidence_variance']:.3f}`",
      f"- Wall time ms mean/variance: `{metrics['wall_time_ms_mean']:.3f}` / `{metrics['wall_time_ms_variance']:.3f}`",
      f"- Blocking gate: `{summary['governance']['blocking_gate']}`",
      "",
      "## Snapping Rules",
      "",
      "- Expression equivalence: `{}`".format(
          summary["snapping_rules"]["expression_equivalence"]),
      "- Parameter RMSE tolerance: `{:.3f}`".format(
          summary["snapping_rules"]["parameter_rmse_tolerance"]),
      "- Numerical sample domain: `{}`".format(
          summary["snapping_rules"]["numerical_equivalence"]["sample_domain"]),
      "- Numerical max abs tolerance: `{:.6e}`".format(
          summary["snapping_rules"]["numerical_equivalence"]
          ["max_abs_error_tolerance"]),
      "- Numerical final-loss tolerance: `{:.6e}`".format(
          summary["snapping_rules"]["numerical_equivalence"]
          ["final_loss_tolerance"]),
      "- Indeterminate state: `{}`".format(
          summary["snapping_rules"]["indeterminate_state"]),
      "",
      "## Task Aggregates",
      "",
      "| Task | Runs | Recovery | Snap | Loss mean | Loss variance | RMSE mean | RMSE variance | Failures |",
      "| ---- | ---- | -------- | ---- | --------- | ------------- | --------- | ------------- | -------- |",
  ]
  for task_metrics in summary["task_metrics"]:
    metrics_row = task_metrics["metrics"]
    lines.append(
        "| `{task}` | `{runs}` | `{recovery:.3f}` | `{snap:.3f}` | "
        "`{loss_mean:.6e}` | `{loss_var:.6e}` | `{rmse_mean:.6e}` | "
        "`{rmse_var:.6e}` | `{failures}` |".format(
            task=task_metrics["name"],
            runs=metrics_row["runs"],
            recovery=metrics_row["recovery_rate"],
            snap=metrics_row["snap_to_symbolic_rate"],
            loss_mean=metrics_row["final_loss_mean"],
            loss_var=metrics_row["final_loss_variance"],
            rmse_mean=metrics_row["param_rmse_mean"],
            rmse_var=metrics_row["param_rmse_variance"],
            failures=metrics_row["failure_summary"]["failed_run_count"],
        ))
  lines.extend([
      "",
      "## Per-task Highlights",
      "",
  ])
  for task_metrics in summary["task_metrics"]:
    metrics_row = task_metrics["metrics"]
    lines.extend([
        f"### {task_metrics['name']}",
        "",
        f"- Template: `{task_metrics['template']}`",
        f"- Best run: `{format_run_reference(metrics_row['best_run'])}`",
        f"- Worst run: `{format_run_reference(metrics_row['worst_run'])}`",
        "- Snapping states: `{}`".format(metrics_row["snapping_state_counts"]),
        "- Failure summary: `{}`".format(
            metrics_row["failure_summary"]["by_reason"]),
    ])
    examples = metrics_row["failure_summary"]["examples"]
    if examples:
      lines.append("- Failure examples:")
      lines.extend(f"  - `{format_run_reference(example)}`"
                   for example in examples)
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


def parse_seeds(seed_args: Sequence[int] | None) -> List[int]:
  """Resolve an ordered deterministic seed set from CLI arguments."""
  if not seed_args:
    return list(DEFAULT_SEEDS)
  ordered: List[int] = []
  for seed in seed_args:
    if seed not in ordered:
      ordered.append(seed)
  return ordered


def main() -> int:
  """Run the SR research benchmark CLI."""
  parser = argparse.ArgumentParser()
  parser.add_argument("--samples", type=int, default=41)
  parser.add_argument("--seed", dest="seeds", type=int, action="append")
  parser.add_argument("--steps", type=int, default=80)
  parser.add_argument("--output-json",
                      default="target/sr-research-benchmark.json")
  parser.add_argument("--output-md", default="target/sr-research-benchmark.md")
  args = parser.parse_args()

  if args.steps <= 0:
    raise SystemExit("--steps must be positive")

  seeds = parse_seeds(args.seeds)
  summary = build_summary(args.samples, seeds, args.steps)
  json_text = json.dumps(summary, indent=2, sort_keys=True)
  write_text(resolve_path(args.output_json), lambda: json_text + "\n")
  write_text(resolve_path(args.output_md), lambda: render_markdown(summary))
  print(json_text)
  print(
      "[sr-research] tasks="
      f"{len(summary['tasks'])} seeds={len(summary['seed_set'])} depths=2..6 recovery_rate="
      f"{summary['metrics']['recovery_rate']:.3f} snap_to_symbolic_rate="
      f"{summary['metrics']['snap_to_symbolic_rate']:.3f} nan_overflow_incidence="
      f"{summary['metrics']['nan_overflow_incidence_mean']:.3f}")
  return 0


if __name__ == "__main__":
  sys.exit(main())
