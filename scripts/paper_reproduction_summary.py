#!/usr/bin/env python3
"""Build a machine-readable paper-basis reproduction summary."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, Iterable, List, Tuple

ROOT = pathlib.Path(__file__).resolve().parents[1]
SUMMARY_SCHEMA = "eml-rs.paper-reproduction-summary.v2"
SUMMARY_SCHEMA_VERSION = 2
CATALOG_SECTIONS = ("constants", "unaryFunctions", "binaryOperations")
REPLAY_TEST_ANCHOR = (
    "tests/paper_reproduction.rs:paper_basis_verify_base_set_witness_chains_replay"
)
SAMPLE_REGIONS = [
    "positive-real-axis",
    "negative-real-axis",
    "zero-neighborhood",
    "complex-plane",
]


def resolve_path(path: str) -> pathlib.Path:
  """Resolve a user path relative to the repository root."""
  candidate = pathlib.Path(path)
  if candidate.is_absolute():
    return candidate
  return ROOT / candidate


def load_catalog(path: pathlib.Path) -> Dict[str, Any]:
  """Load the paper-basis catalog JSON file."""
  if not path.exists():
    raise SystemExit(f"missing catalog: {path}")
  return json.loads(path.read_text(encoding="utf-8"))


def iter_catalog_entries(
    catalog: Dict[str, Any]) -> Iterable[Tuple[str, Dict[str, Any]]]:
  """Yield all first-class paper-basis catalog entries with section names."""
  for section in CATALOG_SECTIONS:
    for entry in catalog.get(section, []):
      yield section, entry


def status_counts(catalog: Dict[str, Any]) -> Dict[str, int]:
  """Count catalog entries by coverage status."""
  counts = {"covered": 0, "partial": 0, "missing": 0, "unknown": 0}
  for _, entry in iter_catalog_entries(catalog):
    status = str(entry.get("status", "unknown"))
    counts[status if status in counts else "unknown"] += 1
  return counts


def coverage_ratio(counts: Dict[str, int]) -> float:
  """Return the covered-entry ratio for a status-count map."""
  total = sum(counts.values())
  if total == 0:
    return 0.0
  return counts.get("covered", 0) / total


def source_formula_for_entry(section: str, entry: Dict[str, Any]) -> str:
  """Infer the source formula used for a replayed catalog entry."""
  name = str(entry.get("name", "unknown"))
  if name == "log_base":
    return "log_x(x, y)"
  if display := entry.get("display"):
    return str(display)
  if section == "unaryFunctions":
    if name == "minus":
      return "-x"
    if name == "log":
      return "log(x)"
    return f"{name}(x)"
  return name


def entry_has_replay_anchor(entry: Dict[str, Any]) -> bool:
  """Return whether a catalog entry is marked as replayed by the Rust harness."""
  anchors = entry.get("testAnchors", [])
  return isinstance(anchors, list) and REPLAY_TEST_ANCHOR in anchors


def status_detail(section: str, entry: Dict[str, Any]) -> Dict[str, Any]:
  """Build a compact status-detail object for summary artifacts."""
  return {
      "section": section,
      "name": entry.get("name"),
      "display": entry.get("display"),
      "status": entry.get("status", "unknown"),
      "notes": entry.get("notes"),
  }


def entries_with_status(catalog: Dict[str, Any],
                        statuses: set[str]) -> List[Dict[str, Any]]:
  """Collect catalog entries whose status is in the requested set."""
  return [
      status_detail(section, entry)
      for section, entry in iter_catalog_entries(catalog)
      if str(entry.get("status", "unknown")) in statuses
  ]


def witness_provenance_summary(catalog: Dict[str, Any]) -> Dict[str, int]:
  """Count catalog witness provenance sources."""
  counts: Dict[str, int] = {}
  for _, entry in iter_catalog_entries(catalog):
    witness = entry.get("witness")
    source = "without_witness"
    if isinstance(witness, dict):
      source = str(witness.get("source", "unknown"))
    counts[source] = counts.get(source, 0) + 1
  return counts


def build_replayed_witnesses(catalog: Dict[str, Any]) -> List[Dict[str, Any]]:
  """Build replayed witness rows from catalog test anchors."""
  out: List[Dict[str, Any]] = []
  for section, entry in iter_catalog_entries(catalog):
    if not entry_has_replay_anchor(entry):
      continue
    witness = entry.get("witness") if isinstance(entry.get("witness"),
                                                 dict) else {}
    enriched = {
        "name": entry.get("name"),
        "catalog_section": section,
        "catalog_name": entry.get("name"),
        "source_formula": source_formula_for_entry(section, entry),
        "witness_formula": witness.get("formula"),
        "witness_source": witness.get("source"),
        "witness_confidence": witness.get("confidence"),
        "catalog_status": entry.get("status", "unknown"),
        "catalog_category": entry.get("category"),
        "catalog_has_witness": bool(witness),
        "test_anchor": REPLAY_TEST_ANCHOR,
    }
    out.append(enriched)
  return out


def build_summary(catalog_path: pathlib.Path) -> Dict[str, Any]:
  """Build the complete paper reproduction summary."""
  catalog = load_catalog(catalog_path)
  replayed = build_replayed_witnesses(catalog)
  counts = status_counts(catalog)
  covered_ratio = coverage_ratio(counts)
  all_replayed_covered = all(item["catalog_status"] == "covered"
                             and item["catalog_has_witness"]
                             for item in replayed)
  no_missing_replayed = all(
      item["catalog_status"] not in {"missing", "missing-entry", "unknown"}
      and item["catalog_has_witness"] for item in replayed)

  return {
      "schema": SUMMARY_SCHEMA,
      "schema_version": SUMMARY_SCHEMA_VERSION,
      "generated_at": datetime.now(timezone.utc).isoformat(),
      "catalog": str(catalog_path),
      "catalog_schema": catalog.get("schema"),
      "ci_mode": "non-blocking-artifact-first",
      "harness": {
          "rust_test": "tests/paper_reproduction.rs",
          "test_command": "cargo test --test paper_reproduction",
          "comparison":
          "pure_eml_witness vs lowering_result vs source_reference",
          "sample_regions": SAMPLE_REGIONS,
      },
      "catalog_status_counts": counts,
      "catalog_coverage_ratio": covered_ratio,
      "missing_or_partial_entries": entries_with_status(catalog,
                                                         {"missing", "partial"}),
      "witness_provenance_summary": witness_provenance_summary(catalog),
      "replayed_witnesses": replayed,
      "acceptance": {
          "all_replayed_witnesses_are_catalog_covered": all_replayed_covered,
          "no_missing_replayed_witnesses": no_missing_replayed,
          "catalog_covered_ratio": covered_ratio,
          "blocking_gate": False,
          "next_gate_decision": "promote after artifact history is stable",
      },
  }


def render_markdown(summary: Dict[str, Any]) -> str:
  """Render a human-readable Markdown summary."""
  lines = [
      "# Paper Reproduction Summary",
      "",
      f"- Schema: `{summary['schema']}`",
      f"- Schema version: `{summary['schema_version']}`",
      f"- Generated at: `{summary['generated_at']}`",
      f"- CI mode: `{summary['ci_mode']}`",
      f"- Rust test: `{summary['harness']['rust_test']}`",
      f"- Comparison: `{summary['harness']['comparison']}`",
      f"- Blocking gate: `{summary['acceptance']['blocking_gate']}`",
      f"- Catalog covered ratio: `{summary['catalog_coverage_ratio']:.4f}`",
      "",
      "## Sample Regions",
      "",
  ]
  lines.extend(f"- `{region}`"
               for region in summary["harness"]["sample_regions"])
  lines.extend([
      "",
      "## Replayed Witnesses",
      "",
      "| Name | Source | Witness | Catalog status | Provenance |",
      "| ---- | ------ | ------- | -------------- | ---------- |",
  ])
  for witness in summary["replayed_witnesses"]:
    lines.append(
        "| `{name}` | `{source}` | `{witness}` | `{status}` | `{provenance}` |".
        format(
            name=witness["name"],
            source=witness["source_formula"],
            witness=witness["witness_formula"],
            status=witness["catalog_status"],
            provenance=witness["witness_source"],
        ))
  lines.extend([
      "",
      "## Catalog Status Counts",
      "",
  ])
  for status, count in sorted(summary["catalog_status_counts"].items()):
    lines.append(f"- `{status}`: `{count}`")
  lines.extend([
      "",
      "## Missing Or Partial Entries",
      "",
  ])
  if summary["missing_or_partial_entries"]:
    for entry in summary["missing_or_partial_entries"]:
      lines.append("- `{section}.{name}`: `{status}`".format(
          section=entry["section"],
          name=entry["name"],
          status=entry["status"],
      ))
  else:
    lines.append("- None")
  lines.extend([
      "",
      "## Witness Provenance",
      "",
  ])
  for source, count in sorted(summary["witness_provenance_summary"].items()):
    lines.append(f"- `{source}`: `{count}`")
  lines.append("")
  return "\n".join(lines)


def write_text(path: pathlib.Path, text: str) -> None:
  """Create parent directories and write UTF-8 text."""
  path.parent.mkdir(parents=True, exist_ok=True)
  path.write_text(text, encoding="utf-8")


def main() -> int:
  """Run the paper reproduction summary CLI."""
  parser = argparse.ArgumentParser()
  parser.add_argument("--catalog", default="docs/paper-basis-catalog.json")
  parser.add_argument("--output-json",
                      default="target/paper-reproduction-summary.json")
  parser.add_argument("--output-md",
                      default="target/paper-reproduction-summary.md")
  parser.add_argument("--require-all-covered", action="store_true")
  parser.add_argument("--require-min-covered-ratio", type=float)
  parser.add_argument("--require-no-missing-replayed", action="store_true")
  args = parser.parse_args()

  if args.require_min_covered_ratio is not None and not 0 <= args.require_min_covered_ratio <= 1:
    parser.error("--require-min-covered-ratio must be between 0 and 1")

  summary = build_summary(resolve_path(args.catalog))
  json_text = json.dumps(summary, indent=2, sort_keys=True)
  md_text = render_markdown(summary)
  write_text(resolve_path(args.output_json), json_text + "\n")
  write_text(resolve_path(args.output_md), md_text)

  print(json_text)
  print(
      "[paper-reproduction] replayed_witnesses="
      f"{len(summary['replayed_witnesses'])} "
      "all_catalog_covered="
      f"{summary['acceptance']['all_replayed_witnesses_are_catalog_covered']}")
  failures = []
  if args.require_all_covered:
    if not summary["acceptance"]["all_replayed_witnesses_are_catalog_covered"]:
      failures.append("not all replayed witnesses are catalog covered")
  if args.require_no_missing_replayed:
    if not summary["acceptance"]["no_missing_replayed_witnesses"]:
      failures.append("some replayed witnesses are missing or lack witness metadata")
  if args.require_min_covered_ratio is not None:
    actual = summary["acceptance"]["catalog_covered_ratio"]
    if actual < args.require_min_covered_ratio:
      failures.append(
          "catalog covered ratio "
          f"{actual:.4f} is below {args.require_min_covered_ratio:.4f}")
  if failures:
    for failure in failures:
      print(f"[paper-reproduction] gate failed: {failure}", file=sys.stderr)
    return 1
  return 0


if __name__ == "__main__":
  sys.exit(main())
