#!/usr/bin/env python3
"""Scan BID console log for WARN (report) / ERROR+panic (fail).

Usage (from workspace cell root):
  python3 jenkins/check_log_severity.py --bid aarch64/rk3568
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

from board_flow import bid_log_name

ANSI_RE = re.compile(
    r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*\x07|\x1b[()][\w0-9]?|\r"
)
WARN_RE = re.compile(r"\[\s*WARN\b")
ERROR_RE = re.compile(r"\[\s*(ERROR|ERR|CRIT)\b")
PANIC_RE = re.compile(
    r"\b(Kernel panic|panicked at|PANIC|Oops:|BUG:)\b", re.IGNORECASE
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bid", required=True, help="arch/board BID")
    args = parser.parse_args()

    path = Path("logs") / bid_log_name(args.bid)
    if not path.is_file():
        print(f"[logcheck] missing {path.as_posix()}, skip")
        return 0

    print(f"[logcheck] file: {path.as_posix()}")
    text = ANSI_RE.sub("", path.read_text(encoding="utf-8", errors="replace"))
    warns, errors, panics = [], [], []
    for n, line in enumerate(text.splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        hit = f"{path.as_posix()}:{n}: {line}"
        if PANIC_RE.search(line):
            panics.append(hit)
        elif ERROR_RE.search(line):
            errors.append(hit)
        elif WARN_RE.search(line):
            warns.append(hit)

    for label, items in (("WARN", warns), ("ERROR", errors), ("PANIC", panics)):
        if items:
            print(f"[logcheck] {len(items)} {label}")
            for item in items[:20]:
                print(f"  {item}")
            if len(items) > 20:
                print(f"  ... {len(items) - 20} more")

    if errors or panics:
        print("[logcheck] FAILED")
        return 1
    print("[logcheck] OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
