#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Create synthetic A/B/C result files to verify diff and report tooling."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


SAMPLE = {
    "t_read_write": ("PASS", "PASS", "PASS"),
    "t_mmap_munmap": ("PASS", "PASS", "FAIL"),
    "t_ext2_rw": ("FAIL", "PASS", "FAIL"),
    "t_tcp_loopback": ("PASS", "FAIL", "FAIL"),
    "t_ptrace_attach": ("SKIP", "PASS", "SKIP"),
}


def write_env(out: Path, env: str, index: int) -> None:
    rows = []
    for name, statuses in SAMPLE.items():
        status = statuses[index]
        rows.append(
            {
                "name": name,
                "layer": "fixture",
                "status": status,
                "exit_code": 0 if status == "PASS" else (77 if status == "SKIP" else 1),
                "duration_ms": 1,
                "output": f"{name}: {status} (synthetic fixture)",
            }
        )
    data = {
        "schema_version": 1,
        "metadata": {
            "env_id": env,
            "measurement_status": "synthetic-tool-test",
            "warning": "not real experiment data",
        },
        "results": rows,
    }
    path = out / f"env_{env.lower()}.json"
    with path.open("w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    args.out.mkdir(parents=True, exist_ok=True)
    for idx, env in enumerate(["A", "B", "C"]):
        write_env(args.out, env, idx)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
