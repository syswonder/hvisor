#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Repeat-boot stability harness for Env A (Asterinas on QEMU/KVM).

Boots the autorun ISO N times, collecting results.json each round, and reports
whether the pass/fail/skip counts are stable across reboots (the runtime
stability criterion). Writes a stability summary JSON.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main(argv):
    p = argparse.ArgumentParser()
    p.add_argument("--iso", type=Path, default=ROOT / "_build" / "aster-envA-autorun.iso")
    p.add_argument("--rounds", type=int, default=3)
    p.add_argument("--env-id", default="A")
    p.add_argument("--out-dir", type=Path, default=ROOT / "results" / "stability")
    p.add_argument("--max-seconds", type=int, default=2400)
    args = p.parse_args(argv)

    args.out_dir.mkdir(parents=True, exist_ok=True)
    rounds = []
    for i in range(1, args.rounds + 1):
        rj = args.out_dir / f"round_{i}.json"
        cl = args.out_dir / f"round_{i}.console.log"
        print(f"=== stability round {i}/{args.rounds} ===")
        t0 = time.monotonic()
        rc = subprocess.call([
            sys.executable, str(ROOT / "harness" / "boot_collect.py"),
            "--iso", str(args.iso), "--env-id", args.env_id,
            "--out", str(rj), "--console-log", str(cl),
            "--cpu", "Icelake-Server", "--smp", "4", "--mem", "4G",
            "--max-seconds", str(args.max_seconds),
        ])
        elapsed = time.monotonic() - t0
        rec = {"round": i, "collect_rc": rc, "wall_seconds": round(elapsed, 1)}
        if rj.exists():
            try:
                d = json.loads(rj.read_text())
                rec["summary"] = d.get("summary", {})
                rec["boot_seconds"] = d.get("metadata", {}).get("wall_clock_seconds")
            except Exception as e:  # pragma: no cover
                rec["error"] = str(e)
        rounds.append(rec)
        print(f"  round {i}: rc={rc} summary={rec.get('summary')} ({elapsed:.0f}s)")

    summaries = [r.get("summary", {}) for r in rounds if r.get("summary")]
    stable = len({json.dumps(s, sort_keys=True) for s in summaries}) <= 1 and len(summaries) == args.rounds
    out = {
        "schema_version": 1,
        "rounds": rounds,
        "stable": stable,
        "note": "stable=true means identical pass/fail/skip counts across every successful boot",
    }
    (args.out_dir / "stability_summary.json").write_text(json.dumps(out, indent=2) + "\n")
    print(f"=== stability: {'STABLE' if stable else 'VARIANCE DETECTED'} across {len(summaries)}/{args.rounds} boots ===")
    print(f"summary -> {args.out_dir / 'stability_summary.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
