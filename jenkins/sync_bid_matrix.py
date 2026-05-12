#!/usr/bin/env python3
"""Sync BID matrix values in Jenkinsfile from platform directory."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def collect_bids(platform_dir: Path) -> list[str]:
    if not platform_dir.is_dir():
        raise FileNotFoundError(f"platform dir not found: {platform_dir}")

    bids: list[str] = []
    for arch_dir in sorted(p for p in platform_dir.iterdir() if p.is_dir()):
        for board_dir in sorted(p for p in arch_dir.iterdir() if p.is_dir()):
            bids.append(f"{arch_dir.name}/{board_dir.name}")
    return bids


def find_bid_values_block(lines: list[str]) -> tuple[int, int, str]:
    """Return (start_idx, end_idx, values_indent). end_idx points to ')' line."""
    bid_line_idx = None
    for i, line in enumerate(lines):
        if "name 'BID'" in line:
            bid_line_idx = i
            break
    if bid_line_idx is None:
        raise ValueError("cannot find `name 'BID'` in Jenkinsfile")

    values_idx = None
    values_indent = ""
    for i in range(bid_line_idx + 1, len(lines)):
        stripped = lines[i].strip()
        if not stripped:
            continue
        if stripped.startswith("values("):
            values_idx = i
            values_indent = lines[i][: len(lines[i]) - len(lines[i].lstrip(" "))]
            break
        raise ValueError("cannot find `values(` right after `name 'BID'`")
    if values_idx is None:
        raise ValueError("cannot find `values(` in BID axis")

    end_idx = None
    for i in range(values_idx + 1, len(lines)):
        if lines[i].strip() == ")":
            end_idx = i
            break
    if end_idx is None:
        raise ValueError("cannot find closing `)` for BID values block")

    return values_idx, end_idx, values_indent


def build_value_lines(bids: list[str], values_indent: str) -> list[str]:
    item_indent = values_indent + "    "
    return [f"{item_indent}'{bid}',\n" for bid in bids]


def sync_jenkinsfile(jenkinsfile: Path, bids: list[str], write: bool) -> bool:
    original = jenkinsfile.read_text(encoding="utf-8")
    lines = original.splitlines(keepends=True)

    values_idx, end_idx, values_indent = find_bid_values_block(lines)
    new_lines = build_value_lines(bids, values_indent)

    updated_lines = lines[: values_idx + 1] + new_lines + lines[end_idx:]
    updated = "".join(updated_lines)
    changed = updated != original

    if changed and write:
        jenkinsfile.write_text(updated, encoding="utf-8")
    return changed


def parse_args() -> argparse.Namespace:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(
        description="Sync Jenkinsfile BID matrix from platform directories."
    )
    parser.add_argument(
        "--platform-dir",
        type=Path,
        default=repo_root / "platform",
        help="Path to platform root directory (default: %(default)s)",
    )
    parser.add_argument(
        "--jenkinsfile",
        type=Path,
        default=repo_root / "Jenkinsfile",
        help="Path to Jenkinsfile (default: %(default)s)",
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="Update Jenkinsfile. Without this flag, only check diff.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit with code 1 when Jenkinsfile is out of sync.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="Print all discovered BID values and exit.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    bids = collect_bids(args.platform_dir)
    if args.list:
        print("\n".join(bids))
        return 0

    changed = sync_jenkinsfile(args.jenkinsfile, bids, write=args.update)

    if changed:
        if args.update:
            print(f"updated {args.jenkinsfile} with {len(bids)} bids")
        else:
            print(f"{args.jenkinsfile} is out of sync ({len(bids)} bids expected)")
    else:
        print(f"{args.jenkinsfile} is up to date ({len(bids)} bids)")

    if args.check and changed:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
