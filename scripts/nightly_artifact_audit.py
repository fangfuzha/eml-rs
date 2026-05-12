#!/usr/bin/env python3
"""Audit downloaded nightly paper-reproduction and SR research artifacts."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, Iterable, List

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUDIT_SCHEMA = "eml-rs.nightly-artifact-audit.v1"
AUDIT_SCHEMA_VERSION = 1
PAPER_SUMMARY_SCHEMA = "eml-rs.paper-reproduction-summary.v2"
SR_SUMMARY_SCHEMA = "eml-rs.sr-research-benchmark.v2"
SR_ARTIFACT_POLICY = "nightly-and-workflow-dispatch-non-blocking"


def resolve_path(path: str) -> pathlib.Path:
  """Resolve a user-provided path relative to the repository root."""
  candidate = pathlib.Path(path)
  if candidate.is_absolute():
    return candidate
  return ROOT / candidate


def display_path(path: pathlib.Path) -> str:
  """Return a stable, repository-relative display path when possible."""
  try:
    return path.resolve().relative_to(ROOT).as_posix()
  except ValueError:
    return str(path)


def load_json(path: pathlib.Path) -> Dict[str, Any]:
  """Load one JSON artifact as an object."""
  if not path.exists():
    raise SystemExit(f"missing artifact: {path}")
  try:
    data = json.loads(path.read_text(encoding="utf-8"))
  except json.JSONDecodeError as error:
    raise SystemExit(f"invalid JSON artifact {path}: {error}") from error
  if not isinstance(data, dict):
    raise SystemExit(f"artifact must be a JSON object: {path}")
  return data


def unique_paths(paths: Iterable[pathlib.Path]) -> List[pathlib.Path]:
  """Return paths in first-seen order with duplicates removed."""
  seen: set[pathlib.Path] = set()
  out: List[pathlib.Path] = []
  for path in paths:
    resolved = path.resolve()
    if resolved in seen:
      continue
    seen.add(resolved)
    out.append(path)
  return out


def collect_named_json(
    roots: Iterable[pathlib.Path], filename: str) -> List[pathlib.Path]:
  """Collect artifacts with a fixed filename under each root directory."""
  paths: List[pathlib.Path] = []
  for root in roots:
    if root.is_file() and root.name == filename:
      paths.append(root)
      continue
    if root.is_dir():
      paths.extend(sorted(root.rglob(filename)))
  return unique_paths(paths)


def get_nested(data: Dict[str, Any], keys: Iterable[str]) -> Any:
  """Read a nested JSON value from a dictionary, returning None if absent."""
  current: Any = data
  for key in keys:
    if not isinstance(current, dict) or key not in current:
      return None
    current = current[key]
  return current


def is_zero(value: Any) -> bool:
  """Return whether a JSON value is numerically zero."""
  return isinstance(value, (int, float)) and value == 0


def non_empty_collection(value: Any) -> bool:
  """Return whether a JSON value is a non-empty object or array."""
  return isinstance(value, (dict, list)) and len(value) > 0


def audit_paper_artifact(path: pathlib.Path,
                         require_all_covered: bool) -> Dict[str, Any]:
  """Audit one paper reproduction summary artifact."""
  data = load_json(path)
  status_counts = data.get("catalog_status_counts")
  acceptance = data.get("acceptance")
  missing_or_partial = data.get("missing_or_partial_entries")
  coverage_ratio = data.get("catalog_coverage_ratio")
  checks = {
      "schema_v2": data.get("schema") == PAPER_SUMMARY_SCHEMA,
      "catalog_schema_present": isinstance(data.get("catalog_schema"), str),
      "status_counts_present": isinstance(status_counts, dict),
      "acceptance_present": isinstance(acceptance, dict),
      "replayed_witnesses_present": isinstance(data.get("replayed_witnesses"), list),
      "non_blocking_summary": data.get("ci_mode") == "non-blocking-artifact-first",
  }
  if isinstance(status_counts, dict):
    checks["missing_zero"] = is_zero(status_counts.get("missing"))
    checks["partial_zero"] = is_zero(status_counts.get("partial"))
  if isinstance(acceptance, dict):
    checks["blocking_gate_false"] = acceptance.get("blocking_gate") is False
    checks["replayed_witnesses_catalog_covered"] = (
        acceptance.get("all_replayed_witnesses_are_catalog_covered") is True)
    checks["no_missing_replayed_witnesses"] = (
        acceptance.get("no_missing_replayed_witnesses") is True)
  if require_all_covered:
    checks["coverage_ratio_one"] = coverage_ratio == 1.0
    checks["missing_or_partial_empty"] = missing_or_partial == []

  passed = all(checks.values())
  return {
      "path": display_path(path),
      "schema": data.get("schema"),
      "generated_at": data.get("generated_at"),
      "catalog_schema": data.get("catalog_schema"),
      "catalog_status_counts": status_counts,
      "catalog_coverage_ratio": coverage_ratio,
      "checks": checks,
      "passed": passed,
  }


def audit_sr_artifact(path: pathlib.Path,
                      require_non_blocking: bool) -> Dict[str, Any]:
  """Audit one symbolic-regression research summary artifact."""
  data = load_json(path)
  governance = data.get("governance")
  tasks = data.get("tasks")
  runs = data.get("runs")
  task_metrics = data.get("task_metrics")
  checks = {
      "schema_v2": data.get("schema") == SR_SUMMARY_SCHEMA,
      "governance_present": isinstance(governance, dict),
      "tasks_present": isinstance(tasks, list) and len(tasks) > 0,
      "runs_present": isinstance(runs, list) and len(runs) > 0,
        "task_metrics_present": non_empty_collection(task_metrics),
      "snapping_rules_present": isinstance(data.get("snapping_rules"), dict),
      "failure_summary_present": isinstance(
          get_nested(data, ("metrics", "failure_summary")), dict),
  }
  if require_non_blocking and isinstance(governance, dict):
    checks["artifact_policy_non_blocking"] = (
        governance.get("artifact_policy") == SR_ARTIFACT_POLICY)
    checks["blocking_gate_false"] = governance.get("blocking_gate") is False
    checks["track_is_sr_research"] = (
        governance.get("track") == "symbolic-regression-research")

  passed = all(checks.values())
  return {
      "path": display_path(path),
      "schema": data.get("schema"),
      "generated_at": data.get("generated_at"),
      "task_count": len(tasks) if isinstance(tasks, list) else None,
      "run_count": len(runs) if isinstance(runs, list) else None,
      "task_metric_count": len(task_metrics) if isinstance(task_metrics, list)
      else len(task_metrics) if isinstance(task_metrics, dict) else None,
      "artifact_policy": get_nested(data, ("governance", "artifact_policy")),
      "blocking_gate": get_nested(data, ("governance", "blocking_gate")),
      "checks": checks,
      "passed": passed,
  }


def collect_artifact_paths(args: argparse.Namespace) -> tuple[List[pathlib.Path],
                                                              List[pathlib.Path]]:
  """Collect paper and SR artifact paths from explicit files and roots."""
  roots = [resolve_path(path) for path in args.artifact_root]
  if not roots and (ROOT / "target" / "remote-artifacts").exists():
    roots.append(ROOT / "target" / "remote-artifacts")

  paper_paths = [resolve_path(path) for path in args.paper_json]
  sr_paths = [resolve_path(path) for path in args.sr_json]
  paper_paths.extend(collect_named_json(roots, "paper-reproduction-summary.json"))
  sr_paths.extend(collect_named_json(roots, "sr-research-benchmark.json"))
  return unique_paths(paper_paths), unique_paths(sr_paths)


def build_audit(args: argparse.Namespace) -> Dict[str, Any]:
  """Build a complete artifact audit summary."""
  paper_paths, sr_paths = collect_artifact_paths(args)
  paper_artifacts = [
      audit_paper_artifact(path, args.require_paper_all_covered)
      for path in paper_paths
  ]
  sr_artifacts = [
      audit_sr_artifact(path, args.require_sr_non_blocking)
      for path in sr_paths
  ]
  errors: List[str] = []
  if len(paper_artifacts) < args.require_min_paper_artifacts:
    errors.append(
        f"expected at least {args.require_min_paper_artifacts} paper artifacts, found {len(paper_artifacts)}"
    )
  if len(sr_artifacts) < args.require_min_sr_artifacts:
    errors.append(
        f"expected at least {args.require_min_sr_artifacts} SR artifacts, found {len(sr_artifacts)}"
    )
  errors.extend(
      f"paper artifact failed checks: {artifact['path']}"
      for artifact in paper_artifacts if not artifact["passed"])
  errors.extend(f"SR artifact failed checks: {artifact['path']}"
                for artifact in sr_artifacts if not artifact["passed"])

  return {
      "schema": AUDIT_SCHEMA,
      "schema_version": AUDIT_SCHEMA_VERSION,
      "generated_at": datetime.now(timezone.utc).isoformat(),
      "requirements": {
          "min_paper_artifacts": args.require_min_paper_artifacts,
          "min_sr_artifacts": args.require_min_sr_artifacts,
          "paper_all_covered": args.require_paper_all_covered,
          "sr_non_blocking": args.require_sr_non_blocking,
      },
      "artifact_counts": {
          "paper": len(paper_artifacts),
          "sr": len(sr_artifacts),
      },
      "paper_artifacts": paper_artifacts,
      "sr_artifacts": sr_artifacts,
      "passed": not errors,
      "errors": errors,
  }


def render_markdown(audit: Dict[str, Any]) -> str:
  """Render an artifact audit as Markdown."""
  lines = [
      "# Nightly Artifact Audit",
      "",
      f"- Schema: `{audit['schema']}`",
      f"- Generated at: `{audit['generated_at']}`",
      f"- Passed: `{audit['passed']}`",
      f"- Paper artifacts: `{audit['artifact_counts']['paper']}`",
      f"- SR artifacts: `{audit['artifact_counts']['sr']}`",
      "",
      "## Paper Artifacts",
      "",
  ]
  for artifact in audit["paper_artifacts"]:
    counts = artifact.get("catalog_status_counts") or {}
    lines.extend([
        f"- `{artifact['path']}`",
        f"  - Schema: `{artifact.get('schema')}`",
        f"  - Coverage ratio: `{artifact.get('catalog_coverage_ratio')}`",
        f"  - Covered/missing/partial: `{counts.get('covered')}/{counts.get('missing')}/{counts.get('partial')}`",
        f"  - Passed: `{artifact['passed']}`",
    ])
  lines.extend(["", "## SR Artifacts", ""])
  for artifact in audit["sr_artifacts"]:
    lines.extend([
        f"- `{artifact['path']}`",
        f"  - Schema: `{artifact.get('schema')}`",
        f"  - Tasks/runs/task metrics: `{artifact.get('task_count')}/{artifact.get('run_count')}/{artifact.get('task_metric_count')}`",
        f"  - Artifact policy: `{artifact.get('artifact_policy')}`",
        f"  - Blocking gate: `{artifact.get('blocking_gate')}`",
        f"  - Passed: `{artifact['passed']}`",
    ])
  if audit["errors"]:
    lines.extend(["", "## Errors", ""])
    lines.extend(f"- {error}" for error in audit["errors"])
  lines.append("")
  return "\n".join(lines)


def write_outputs(audit: Dict[str, Any], args: argparse.Namespace) -> None:
  """Write optional JSON and Markdown audit outputs."""
  if args.output_json:
    output_json = resolve_path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(audit, indent=2) + "\n", encoding="utf-8")
  if args.output_md:
    output_md = resolve_path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_md.write_text(render_markdown(audit), encoding="utf-8")


def parse_args(argv: List[str]) -> argparse.Namespace:
  """Parse command line arguments for the artifact audit tool."""
  parser = argparse.ArgumentParser(
      description="Audit downloaded nightly paper/SR artifacts.")
  parser.add_argument("--artifact-root", action="append", default=[])
  parser.add_argument("--paper-json", action="append", default=[])
  parser.add_argument("--sr-json", action="append", default=[])
  parser.add_argument("--require-min-paper-artifacts", type=int, default=1)
  parser.add_argument("--require-min-sr-artifacts", type=int, default=1)
  parser.add_argument("--require-paper-all-covered", action="store_true")
  parser.add_argument("--require-sr-non-blocking", action="store_true")
  parser.add_argument("--output-json")
  parser.add_argument("--output-md")
  parser.add_argument("--print-summary", action="store_true")
  return parser.parse_args(argv)


def main(argv: List[str]) -> int:
  """Run the nightly artifact audit command."""
  args = parse_args(argv)
  audit = build_audit(args)
  write_outputs(audit, args)
  if args.print_summary:
    print(render_markdown(audit))
  if not audit["passed"]:
    for error in audit["errors"]:
      print(f"[artifact-audit] {error}", file=sys.stderr)
    return 1
  return 0


if __name__ == "__main__":
  sys.exit(main(sys.argv[1:]))
