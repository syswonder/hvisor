#!/usr/bin/env python3
"""Read jenkins/ci.yaml for BID and test metadata."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parent.parent
CI_YAML = ROOT / "jenkins" / "ci.yaml"


def normalize_build_args(raw: list[Any] | None) -> dict[str, str]:
    result: dict[str, str] = {}
    for item in raw or []:
        if isinstance(item, dict):
            for key, val in item.items():
                result[str(key)] = str(val)
            continue
        parts = str(item).split("=", 1)
        if len(parts) == 2:
            result[parts[0]] = parts[1]
    return result


def load_ci() -> dict[str, Any]:
    if not CI_YAML.exists():
        raise RuntimeError(f"missing ci config: {CI_YAML}")
    with CI_YAML.open("r", encoding="utf-8") as fh:
        data = yaml.safe_load(fh) or {}
    bids = data.get("bids")
    if not isinstance(bids, list):
        raise RuntimeError("ci.yaml: 'bids' must be a list")
    return data


def list_bids(data: dict[str, Any]) -> list[str]:
    out: list[str] = []
    for item in data.get("bids", []):
        bid = str((item or {}).get("bid", "")).strip()
        if bid:
            out.append(bid)
    return out


def get_bid_entry(data: dict[str, Any], bid: str) -> dict[str, Any]:
    for item in data.get("bids", []):
        entry = item or {}
        if str(entry.get("bid", "")).strip() == bid:
            build_args = normalize_build_args(entry.get("build_args"))
            tests = entry.get("tests") or {}
            mode = str(tests.get("mode", "")).strip()
            cases = tests.get("cases") or []
            if not isinstance(cases, list):
                raise RuntimeError(f"{bid}: tests.cases must be a list")
            return {
                "bid": bid,
                "build_args": build_args,
                "mode": mode,
                "cases": [str(x) for x in cases if str(x).strip()],
                "tests": tests,
            }
    raise RuntimeError(f"bid not found in ci.yaml: {bid}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read jenkins/ci.yaml")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("list-bids", help="List all BID entries")
    get_parser = sub.add_parser("get-bid", help="Get one BID config")
    get_parser.add_argument("--bid", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    data = load_ci()
    if args.command == "list-bids":
        print(json.dumps({"bids": list_bids(data)}))
        return 0
    if args.command == "get-bid":
        print(json.dumps(get_bid_entry(data, args.bid)))
        return 0
    raise RuntimeError(f"unknown command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
