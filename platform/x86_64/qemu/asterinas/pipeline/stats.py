#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
"""Read numeric samples from stdin or files and print percentile statistics."""

from __future__ import annotations

import argparse
import json
import math
import statistics
import sys
from pathlib import Path


PERCENTILES = [50, 90, 95, 99, 99.9]


def parse_samples(paths: list[Path]) -> list[float]:
    streams = [sys.stdin] if not paths else [path.open("r", encoding="utf-8") for path in paths]
    values: list[float] = []
    try:
        for stream in streams:
            for line in stream:
                text = line.strip()
                if not text or text.startswith("#"):
                    continue
                # A line is a sample only if it is entirely a single finite number.
                try:
                    value = float(text)
                except ValueError:
                    continue
                if not math.isfinite(value):
                    continue
                values.append(value)
        return values
    finally:
        if paths:
            for stream in streams:
                stream.close()


def percentile(sorted_values: list[float], p: float) -> float:
    if not sorted_values:
        return math.nan
    if len(sorted_values) == 1:
        return sorted_values[0]
    rank = (p / 100.0) * (len(sorted_values) - 1)
    lo = math.floor(rank)
    hi = math.ceil(rank)
    if lo == hi:
        return sorted_values[lo]
    frac = rank - lo
    return sorted_values[lo] * (1.0 - frac) + sorted_values[hi] * frac


def build_stats(values: list[float]) -> dict[str, float]:
    ordered = sorted(values)
    result: dict[str, float] = {
        "count": len(values),
        "min": ordered[0],
        "max": ordered[-1],
        "mean": statistics.fmean(values),
    }
    if len(values) > 1:
        result["stdev"] = statistics.stdev(values)
    else:
        result["stdev"] = 0.0
    for p in PERCENTILES:
        key = f"p{str(p).replace('.', '_')}"
        result[key] = percentile(ordered, p)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="sample files; stdin is used when omitted")
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument("--unit", default="", help="unit label printed in text mode")
    args = parser.parse_args()

    values = parse_samples(args.paths)
    if not values:
        print("no numeric samples found", file=sys.stderr)
        return 1

    stats = build_stats(values)
    if args.json:
        print(json.dumps(stats, indent=2, sort_keys=True))
        return 0

    suffix = f" {args.unit}" if args.unit else ""
    print(f"count = {int(stats['count'])}")
    for key in ["min", "p50", "p90", "p95", "p99", "p99_9", "max", "mean", "stdev"]:
        print(f"{key:<6} = {stats[key]:.2f}{suffix}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
