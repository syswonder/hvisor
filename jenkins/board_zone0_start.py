#!/usr/bin/env python3
"""Power-cycle a board and boot zone0 via U-Boot; no network/SSH/deploy/zone1."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ci_runner import (
    board_power_off,
    boot_board_zone0_shell,
    build_terminal,
    load_runtime_config,
    logs_dir,
)
from terminal import TerminalCommandError, TerminalTimeoutError


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Boot board zone0 and stop.")
    parser.add_argument("--bid", required=True, help="BID key in jenkins/ci.yaml")
    parser.add_argument(
        "--no-power-cycle",
        action="store_true",
        help="Skip relay power cycle (board already off or at U-Boot).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    cfg = load_runtime_config(args)
    if cfg["mode"] != "board":
        raise SystemExit(f"bid '{args.bid}' is mode={cfg['mode']!r}, board mode required")

    log_path = logs_dir(cfg) / "zone0_console.log"
    log_path.write_text("", encoding="utf-8")
    term = build_terminal(cfg, log_path)
    term.open()
    ok = False
    try:
        boot_board_zone0_shell(cfg, term, power_cycle=not args.no_power_cycle)
        ok = True
        print(
            f"[board_zone0_start] zone0 ready (serial={cfg['serial_port']}, log={log_path})",
            flush=True,
        )
        return 0
    except (TerminalTimeoutError, TerminalCommandError) as exc:
        print(f"[board_zone0_start] failed: {exc}", flush=True)
        return 1
    finally:
        term.close()
        if not ok:
            board_power_off(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
