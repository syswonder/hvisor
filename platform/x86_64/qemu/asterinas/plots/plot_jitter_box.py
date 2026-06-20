#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
# Run from the plots/ directory (imports common_plot).

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt

from common_plot import label_from_path, read_samples


def main() -> int:
    parser = argparse.ArgumentParser(description="Plot timer jitter box plots.")
    parser.add_argument("samples", nargs="+", type=Path, help="raw jitter sample files in ns")
    parser.add_argument("-o", "--output", type=Path, default=Path("results/jitter_box.png"))
    args = parser.parse_args()

    labels: list[str] = []
    groups: list[list[float]] = []
    for path in args.samples:
        labels.append(label_from_path(path))
        groups.append([value / 1000.0 for value in read_samples(path)])

    plt.figure(figsize=(max(7, len(groups) * 1.3), 5))
    plt.boxplot(groups, tick_labels=labels, showfliers=False)
    plt.ylabel("Timer wakeup jitter (us)")
    plt.grid(True, axis="y", alpha=0.3)
    plt.xticks(rotation=20, ha="right")
    plt.tight_layout()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(args.output, dpi=160)
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
