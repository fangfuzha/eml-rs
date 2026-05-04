#!/usr/bin/env python3
"""采集 eml-rs 的冷启动与 RSS 指标，并输出机器可读 JSON。"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import platform
import re
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from typing import Any, Dict, List


ROOT = pathlib.Path(__file__).resolve().parents[1]


def exe_path(name: str) -> pathlib.Path:
    suffix = ".exe" if os.name == "nt" else ""
    return ROOT / "target" / "release" / "examples" / f"{name}{suffix}"


def run_checked(cmd: List[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(cmd, text=True, capture_output=True, **kwargs)
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"command failed with status {proc.returncode}: {' '.join(cmd)}")
    return proc


def build_examples(skip_build: bool) -> None:
    if skip_build:
        return
    run_checked(
        [
            "cargo",
            "build",
            "--release",
            "--example",
            "metrics_probe",
            "--example",
            "pipeline_api",
        ],
        cwd=ROOT,
    )


def measure_cold_start(exe: pathlib.Path, runs: int) -> Dict[str, Any]:
    values_ms: List[float] = []
    for _ in range(runs):
        started = time.perf_counter()
        run_checked([str(exe)], cwd=ROOT)
        values_ms.append((time.perf_counter() - started) * 1000.0)

    return {
        "runs": runs,
        "min_ms": min(values_ms),
        "median_ms": statistics.median(values_ms),
        "max_ms": max(values_ms),
        "samples_ms": values_ms,
    }


def parse_linux_time(stderr: str) -> int | None:
    match = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", stderr)
    if match:
        return int(match.group(1)) * 1024
    return None


def parse_macos_time(stderr: str) -> int | None:
    match = re.search(r"(\d+)\s+maximum resident set size", stderr)
    if match:
        return int(match.group(1))
    return None


def measure_rss(
    exe: pathlib.Path,
    nodes: int,
    samples: int,
    require_rss: bool,
    skip_rss: bool,
) -> Dict[str, Any]:
    if skip_rss:
        return {"supported": False, "skipped": True}

    system = platform.system().lower()
    time_exe = pathlib.Path("/usr/bin/time")
    if not time_exe.exists() or system not in {"linux", "darwin"}:
        if require_rss:
            raise RuntimeError("RSS collection requires /usr/bin/time on Linux or macOS")
        return {"supported": False, "skipped": False}

    if system == "linux":
        cmd = [str(time_exe), "-v", str(exe), "--nodes", str(nodes), "--samples", str(samples)]
    else:
        cmd = [str(time_exe), "-l", str(exe), "--nodes", str(nodes), "--samples", str(samples)]

    proc = run_checked(cmd, cwd=ROOT)
    probe = json.loads(proc.stdout)
    rss_bytes = parse_linux_time(proc.stderr) if system == "linux" else parse_macos_time(proc.stderr)
    if rss_bytes is None:
        raise RuntimeError("could not parse maximum RSS from /usr/bin/time output")

    return {
        "supported": True,
        "skipped": False,
        "target_nodes": nodes,
        "samples": samples,
        "max_rss_bytes": rss_bytes,
        "probe": probe,
    }


def attach_thresholds(
    report: Dict[str, Any],
    max_cold_start_ms: float,
    max_rss_bytes: int,
) -> bool:
    ok = True
    cold = report["cold_start"]
    cold["threshold_ms"] = max_cold_start_ms
    cold["passed"] = cold["median_ms"] <= max_cold_start_ms
    ok = ok and cold["passed"]

    rss = report["rss"]
    if rss.get("supported") and not rss.get("skipped"):
        rss["threshold_bytes"] = max_rss_bytes
        rss["passed"] = int(rss["max_rss_bytes"]) <= max_rss_bytes
        ok = ok and rss["passed"]
    else:
        rss["threshold_bytes"] = max_rss_bytes
        rss["passed"] = None

    report["passed"] = ok
    return ok


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rss-nodes", type=int, default=1_000_000)
    parser.add_argument("--samples", type=int, default=8)
    parser.add_argument("--cold-runs", type=int, default=5)
    parser.add_argument("--max-cold-start-ms", type=float, default=800.0)
    parser.add_argument("--max-rss-bytes", type=int, default=1_073_741_824)
    parser.add_argument("--output", default="target/eml-metrics.json")
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--skip-rss", action="store_true")
    parser.add_argument("--require-rss", action="store_true")
    args = parser.parse_args()

    if args.rss_nodes <= 0 or args.samples <= 0 or args.cold_runs <= 0:
        raise SystemExit("--rss-nodes, --samples 和 --cold-runs 必须大于 0")

    build_examples(args.skip_build)
    cold_exe = exe_path("pipeline_api")
    probe_exe = exe_path("metrics_probe")

    report: Dict[str, Any] = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "host": {
            "system": platform.system(),
            "machine": platform.machine(),
            "python": platform.python_version(),
        },
        "cold_start": measure_cold_start(cold_exe, args.cold_runs),
        "rss": measure_rss(
            probe_exe,
            args.rss_nodes,
            args.samples,
            args.require_rss,
            args.skip_rss,
        ),
    }
    ok = attach_thresholds(report, args.max_cold_start_ms, args.max_rss_bytes)

    output = pathlib.Path(args.output)
    if not output.is_absolute():
        output = ROOT / output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
