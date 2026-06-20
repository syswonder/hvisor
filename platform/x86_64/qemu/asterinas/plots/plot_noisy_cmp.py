#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
# Run from the plots/ directory (sibling plots import common_plot).

from __future__ import annotations

import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Plot quiet/noisy comparison from a JSON file with metrics. "
            "Format: {'metric': {'quiet': value, 'noisy': value}, ...}"
        )
    )
    parser.add_argument("metrics", type=Path)
    parser.add_argument("-o", "--output", type=Path, default=Path("results/noisy_cmp.png"))
    args = parser.parse_args()

    with args.metrics.open("r", encoding="utf-8") as stream:
        data = json.load(stream)
    if not isinstance(data, dict) or not data:
        raise SystemExit("metrics JSON must be a non-empty object")

    names = list(data.keys())
    quiet = [float(data[name]["quiet"]) for name in names]
    noisy = [float(data[name]["noisy"]) for name in names]
    x = list(range(len(names)))
    width = 0.36

    plt.figure(figsize=(max(7, len(names) * 1.5), 5))
    plt.bar([v - width / 2 for v in x], quiet, width=width, label="quiet")
    plt.bar([v + width / 2 for v in x], noisy, width=width, label="noisy")
    plt.xticks(x, names, rotation=15, ha="right")
    plt.ylabel("Metric value")
    plt.grid(True, axis="y", alpha=0.3)
    plt.legend()
    plt.tight_layout()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(args.output, dpi=160)
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
