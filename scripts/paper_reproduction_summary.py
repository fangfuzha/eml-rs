#!/usr/bin/env python3
"""Build a machine-readable paper-basis reproduction summary."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, Iterable, List

ROOT = pathlib.Path(__file__).resolve().parents[1]
SAMPLE_REGIONS = [
    "positive-real-axis",
    "negative-real-axis",
    "zero-neighborhood",
    "complex-plane",
]
REPLAYED_WITNESSES = [
    {
        "name": "exp",
        "catalog_section": "unaryFunctions",
        "catalog_name": "exp",
        "source_formula": "exp(x)",
        "witness_formula": "eml(x, 1)",
    },
    {
        "name": "ln",
        "catalog_section": "unaryFunctions",
        "catalog_name": "log",
        "source_formula": "log(x)",
        "witness_formula": "eml(1, eml(eml(1, x), 1))",
    },
    {
        "name": "add",
        "catalog_section": "binaryOperations",
        "catalog_name": "add",
        "source_formula": "x + y",
        "witness_formula": "x - (-y)",
    },
    {
        "name": "subtract",
        "catalog_section": "binaryOperations",
        "catalog_name": "subtract",
        "source_formula": "x - y",
        "witness_formula": "eml(ln(x), exp(y))",
    },
    {
        "name": "multiply",
        "catalog_section": "binaryOperations",
        "catalog_name": "multiply",
        "source_formula": "x * y",
        "witness_formula": "exp(ln(x) + ln(y))",
    },
    {
        "name": "divide",
        "catalog_section": "binaryOperations",
        "catalog_name": "divide",
        "source_formula": "x / y",
        "witness_formula": "x * (1 / y)",
    },
    {
        "name": "pow",
        "catalog_section": "binaryOperations",
        "catalog_name": "pow",
        "source_formula": "pow(x, y)",
        "witness_formula": "exp(y * ln(x))",
    },
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


def iter_catalog_entries(catalog: Dict[str, Any]) -> Iterable[Dict[str, Any]]:
  """Yield all first-class paper-basis catalog entries."""
  for section in ("constants", "unaryFunctions", "binaryOperations"):
    for entry in catalog.get(section, []):
      yield entry


def find_entry(catalog: Dict[str, Any], section: str,
               name: str) -> Dict[str, Any] | None:
  """Find a named entry in a catalog section."""
  for entry in catalog.get(section, []):
    if entry.get("name") == name:
      return entry
  return None


def status_counts(catalog: Dict[str, Any]) -> Dict[str, int]:
  """Count catalog entries by coverage status."""
  counts = {"covered": 0, "partial": 0, "missing": 0, "unknown": 0}
  for entry in iter_catalog_entries(catalog):
    status = str(entry.get("status", "unknown"))
    counts[status if status in counts else "unknown"] += 1
  return counts


def build_replayed_witnesses(catalog: Dict[str, Any]) -> List[Dict[str, Any]]:
  """Attach catalog status to every replayed witness case."""
  out: List[Dict[str, Any]] = []
  for witness in REPLAYED_WITNESSES:
    entry = find_entry(
        catalog,
        witness["catalog_section"],
        witness["catalog_name"],
    )
    enriched = dict(witness)
    enriched["catalog_status"] = entry.get(
        "status") if entry else "missing-entry"
    enriched["catalog_category"] = entry.get("category") if entry else None
    enriched["catalog_has_witness"] = bool(entry and entry.get("witness"))
    out.append(enriched)
  return out


def build_summary(catalog_path: pathlib.Path) -> Dict[str, Any]:
  """Build the complete paper reproduction summary."""
  catalog = load_catalog(catalog_path)
  replayed = build_replayed_witnesses(catalog)
  all_replayed_covered = all(item["catalog_status"] == "covered"
                             for item in replayed)

  return {
      "schema": "eml-rs.paper-reproduction-summary.v1",
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
      "catalog_status_counts": status_counts(catalog),
      "replayed_witnesses": replayed,
      "acceptance": {
          "all_replayed_witnesses_are_catalog_covered": all_replayed_covered,
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
      f"- Generated at: `{summary['generated_at']}`",
      f"- CI mode: `{summary['ci_mode']}`",
      f"- Rust test: `{summary['harness']['rust_test']}`",
      f"- Comparison: `{summary['harness']['comparison']}`",
      f"- Blocking gate: `{summary['acceptance']['blocking_gate']}`",
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
      "| Name | Source | Witness | Catalog status |",
      "| ---- | ------ | ------- | -------------- |",
  ])
  for witness in summary["replayed_witnesses"]:
    lines.append("| `{name}` | `{source}` | `{witness}` | `{status}` |".format(
        name=witness["name"],
        source=witness["source_formula"],
        witness=witness["witness_formula"],
        status=witness["catalog_status"],
    ))
  lines.extend([
      "",
      "## Catalog Status Counts",
      "",
  ])
  for status, count in sorted(summary["catalog_status_counts"].items()):
    lines.append(f"- `{status}`: `{count}`")
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
  args = parser.parse_args()

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
  if args.require_all_covered:
    return 0 if summary["acceptance"][
        "all_replayed_witnesses_are_catalog_covered"] else 1
  return 0


if __name__ == "__main__":
  sys.exit(main())
