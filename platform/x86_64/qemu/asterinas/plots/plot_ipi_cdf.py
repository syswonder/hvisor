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
    parser = argparse.ArgumentParser(description="Plot IPI ping-pong latency CDFs.")
    parser.add_argument("samples", nargs="+", type=Path, help="raw latency sample files")
    parser.add_argument("-o", "--output", type=Path, default=Path("results/ipi_cdf.png"))
    parser.add_argument("--unit", default="cycles", help="x-axis unit, e.g. cycles or ns")
    args = parser.parse_args()

    plt.figure(figsize=(8, 5))
    for path in args.samples:
        values = sorted(read_samples(path))
        y = [(idx + 1) / len(values) for idx in range(len(values))]
        plt.plot(values, y, label=label_from_path(path), linewidth=1.8)

    plt.xlabel(f"IPI round-trip latency ({args.unit})")
    plt.ylabel("CDF")
    plt.grid(True, alpha=0.3)
    plt.legend()
    plt.tight_layout()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(args.output, dpi=160)
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
