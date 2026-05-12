#!/usr/bin/env python3
"""Discover platform ARCH/BOARD pairs from platform/*/*."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PLATFORM_DIR = ROOT / "platform"


def discover() -> list[dict[str, str]]:
    items: list[dict[str, str]] = []
    if not PLATFORM_DIR.exists():
        return items

    for arch_dir in sorted(PLATFORM_DIR.iterdir()):
        if not arch_dir.is_dir():
            continue
        for board_dir in sorted(arch_dir.iterdir()):
            if not board_dir.is_dir():
                continue
            items.append(
                {
                    "arch": arch_dir.name,
                    "board": board_dir.name,
                    "bid": f"{arch_dir.name}/{board_dir.name}",
                }
            )
    return items


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Discover platform ARCH/BOARD pairs")
    parser.add_argument(
        "--format",
        choices=["json", "lines"],
        default="json",
        help="Output format",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    items = discover()
    if args.format == "lines":
        for item in items:
            print(item["bid"])
        return 0
    print(json.dumps(items))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
