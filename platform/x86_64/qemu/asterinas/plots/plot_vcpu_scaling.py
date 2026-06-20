#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
# Run from the plots/ directory (sibling plots import common_plot).

from __future__ import annotations

import argparse
import csv
from pathlib import Path

import matplotlib.pyplot as plt


def main() -> int:
    parser = argparse.ArgumentParser(description="Plot vCPU scaling from CSV columns: vcpu,quiet,noisy.")
    parser.add_argument("csv_file", type=Path)
    parser.add_argument("-o", "--output", type=Path, default=Path("results/vcpu_scaling.png"))
    parser.add_argument("--ylabel", default="Metric value")
    args = parser.parse_args()

    vcpu: list[int] = []
    quiet: list[float] = []
    noisy: list[float] = []
    with args.csv_file.open("r", encoding="utf-8", newline="") as stream:
        reader = csv.DictReader(stream)
        for row in reader:
            vcpu.append(int(row["vcpu"]))
            quiet.append(float(row["quiet"]))
            noisy.append(float(row["noisy"]))

    if not vcpu:
        raise SystemExit("CSV contains no rows")

    plt.figure(figsize=(7, 5))
    plt.plot(vcpu, quiet, marker="o", label="quiet")
    plt.plot(vcpu, noisy, marker="o", label="noisy")
    plt.xlabel("vCPU count")
    plt.ylabel(args.ylabel)
    plt.xticks(vcpu)
    plt.grid(True, alpha=0.3)
    plt.legend()
    plt.tight_layout()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(args.output, dpi=160)
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
