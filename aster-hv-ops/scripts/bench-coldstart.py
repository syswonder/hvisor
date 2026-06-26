#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Measure the cold-start latency of the Asterinas root zone.

Each run records the host wall-clock time from QEMU launch to a set of boot
milestones, parsed from the serial console. The selftest initramfs powers the
machine off after its checks, so a run terminates on its own. Results (per-run
and min/median/max/mean) are written as JSON.

Usage:
  bench-coldstart.py <iso> [runs] [out.json]

Environment:
  OVMF   override the OVMF firmware path
  OVMF_SEARCH_DIRS   extra search directories, separated by os.pathsep
"""
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import shutil
import statistics
import sys
import time

MARKERS = [
    ("hvisor_start", re.compile(r"Hello, start HVISOR")),
    ("aster_entry", re.compile(r"Entering the Asterinas entry point")),
    ("smp_up", re.compile(r"All application processors started")),
    ("console", re.compile(r"Registered NS16550A")),
    ("rootfs", re.compile(r"rootfs is ready")),
    ("userspace", re.compile(r"FUNCTIONAL VERIFICATION")),
    ("complete", re.compile(r"VERIFICATION COMPLETE")),
]
REQUIRED = [name for name, _ in MARKERS]


# Repo root: aster-hv-ops/scripts/bench-coldstart.py -> three directories up.
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def repo_rel(path):
    """A repo-relative path for stable JSON output."""
    rel = os.path.relpath(os.path.abspath(path), REPO_ROOT)
    return rel if not rel.startswith(os.pardir) else os.path.basename(path)


def find_ovmf():
    if os.environ.get("OVMF"):
        return os.environ["OVMF"]

    roots = [Path(p) for p in os.environ.get("OVMF_SEARCH_DIRS", "").split(os.pathsep) if p]
    qemu = shutil.which("qemu-system-x86_64")
    if qemu:
        roots.append(Path(qemu).resolve().parent.parent / "share")

    for root in roots:
        for rel in (
            "ovmf/OVMF.fd",
            "OVMF/OVMF.fd",
            "OVMF/OVMF_CODE.fd",
            "edk2/ovmf/OVMF_CODE.fd",
        ):
            path = root / rel
            if path.is_file():
                return str(path)
    raise FileNotFoundError("OVMF firmware not found; set OVMF or OVMF_SEARCH_DIRS")


def qemu_argv(iso):
    return [
        "qemu-system-x86_64", "-machine", "q35,kernel-irqchip=split",
        "-cpu", "host,+x2apic,+invtsc,+vmx", "-accel", "kvm",
        "-smp", "4", "-m", "4G", "-bios", find_ovmf(),
        "-nographic", "-serial", "stdio", "-monitor", "none", "-nodefaults",
        "-device", "intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48",
        "-device", "ioh3420,id=pcie.1,chassis=1",
        "-drive", f"file={iso},format=raw,index=0,media=disk",
    ]


def kill(pid):
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.kill(pid, sig)
        except ProcessLookupError:
            return
        for _ in range(20):  # up to ~1 s for the process to reap
            try:
                if os.waitpid(pid, os.WNOHANG)[0]:
                    return
            except ChildProcessError:
                return
            time.sleep(0.05)


def one_run(iso, budget_s=40.0):
    pid, fd = pty.fork()
    if pid == 0:
        os.execvp("qemu-system-x86_64", qemu_argv(iso))
        os._exit(127)
    t0 = time.monotonic()
    hits, buf, deadline = {}, bytearray(), t0 + budget_s
    while time.monotonic() < deadline:
        r, _, _ = select.select([fd], [], [], 0.2)
        if r:
            try:
                d = os.read(fd, 4096)
            except OSError:
                break
            if not d:
                break
            buf += d
            text = bytes(buf).decode("utf-8", "replace")
            for name, rx in MARKERS:
                if name not in hits and rx.search(text):
                    hits[name] = round((time.monotonic() - t0) * 1000, 1)
            if "complete" in hits:
                break
    kill(pid)
    return hits


def stat(runs, key):
    xs = [r[key] for r in runs if key in r]
    if not xs:
        return None
    return {"n": len(xs), "min": min(xs), "median": round(statistics.median(xs), 1),
            "max": max(xs), "mean": round(statistics.fmean(xs), 1)}


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    iso = sys.argv[1]
    runs_n = int(sys.argv[2]) if len(sys.argv) > 2 else 7
    out = sys.argv[3] if len(sys.argv) > 3 else "coldstart_bench.json"

    runs = []
    for i in range(runs_n):
        h = one_run(iso)
        runs.append(h)
        print(f"run {i + 1}: " + " ".join(f"{k}={v}ms" for k, v in h.items()), flush=True)
        time.sleep(0.5)

    for i, h in enumerate(runs):
        missing = [m for m in REQUIRED if m not in h]
        if missing:
            print(f"bench: ERROR, run {i + 1} missing milestone(s): {', '.join(missing)}",
                  file=sys.stderr)
            return 1

    summary = {k: stat(runs, k) for k, _ in MARKERS}
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"iso": repo_rel(iso), "runs_requested": runs_n,
                   "runs": runs, "summary": summary}, f, indent=2)
        f.write("\n")
    print(f"\nwrote {out}")
    base = summary["hvisor_start"]
    for k, _ in MARKERS:
        s = summary[k]
        if s and base:
            print(f"  {k:12s} median={s['median']:9.1f}ms  "
                  f"(+{s['median'] - base['median']:8.1f}ms from hvisor start)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
