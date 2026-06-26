#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Merge A/B/C result JSON files and emit a differential matrix."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from classify import classify, classify_vs_linux, delta, normalize_status
except ImportError:  # pragma: no cover
    from .classify import classify, classify_vs_linux, delta, normalize_status


def load_json(path: Path, allow_missing: bool) -> dict:
    if not path.exists():
        if allow_missing:
            return {
                "metadata": {
                    "source": str(path),
                    "measurement_status": "missing",
                },
                "results": [],
            }
        raise FileNotFoundError(path)
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def index_results(data: dict) -> dict[str, dict]:
    indexed = {}
    for item in data.get("results", []):
        name = item.get("name")
        if not name:
            continue
        indexed[name] = item
    return indexed


def load_catalog(path: Path | None) -> tuple[list[str], dict[str, dict]]:
    if path is None:
        return [], {}
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    rows = data.get("tests", [])
    names = [row["name"] for row in rows if "name" in row]
    meta = {row["name"]: row for row in rows if "name" in row}
    return names, meta


def status_of(index: dict[str, dict], name: str) -> str:
    return normalize_status(index.get(name, {}).get("status"))


def output_of(index: dict[str, dict], name: str) -> str:
    return str(index.get(name, {}).get("output", ""))


def duration_of(index: dict[str, dict], name: str):
    return index.get(name, {}).get("duration_ms")


def build_matrix(args: argparse.Namespace) -> dict:
    env_data = {
        "A": load_json(args.env_a, args.allow_missing),
        "B": load_json(args.env_b, args.allow_missing),
        "C": load_json(args.env_c, args.allow_missing),
    }
    ref_linux_data = None
    if args.ref_linux is not None:
        ref_linux_data = load_json(args.ref_linux, allow_missing=True)
    env_index = {key: index_results(data) for key, data in env_data.items()}
    ref_index = index_results(ref_linux_data) if ref_linux_data else {}
    catalog_names, catalog_meta = load_catalog(args.catalog)

    names = set(catalog_names)
    for idx in env_index.values():
        names.update(idx.keys())
    names.update(ref_index.keys())
    ordered_names = [name for name in catalog_names if name in names]
    ordered_names.extend(sorted(names - set(ordered_names)))

    rows = []
    for name in ordered_names:
        a = status_of(env_index["A"], name)
        b = status_of(env_index["B"], name)
        c = status_of(env_index["C"], name)
        tag, reason = classify(a, b, c)
        meta = catalog_meta.get(name, {})
        row = {
            "name": name,
            "layer": meta.get("layer", env_index["C"].get(name, {}).get("layer", "Unknown")),
            "description": meta.get("description", ""),
            "env_a": a,
            "env_b": b,
            "env_c": c,
            "delta_c_a": delta(c, a),
            "delta_c_b": delta(c, b),
            "attribution": tag,
            "reason": reason,
            "durations_ms": {
                "A": duration_of(env_index["A"], name),
                "B": duration_of(env_index["B"], name),
                "C": duration_of(env_index["C"], name),
            },
            "outputs": {
                "A": output_of(env_index["A"], name)[-1000:],
                "B": output_of(env_index["B"], name)[-1000:],
                "C": output_of(env_index["C"], name)[-1000:],
            },
        }
        if ref_linux_data is not None:
            ref = status_of(ref_index, name)
            vtag, vreason = classify_vs_linux(a, ref)
            row["ref_linux"] = ref
            row["delta_a_linux"] = delta(a, ref)
            row["aster_vs_linux"] = vtag
            row["aster_vs_linux_reason"] = vreason
            row["outputs"]["ref_linux"] = output_of(ref_index, name)[-1000:]
            row["durations_ms"]["ref_linux"] = duration_of(ref_index, name)
        rows.append(row)

    summary = {}
    for row in rows:
        summary[row["attribution"]] = summary.get(row["attribution"], 0) + 1
    for env_key in ["env_a", "env_b", "env_c"]:
        counts = {}
        for row in rows:
            counts[row[env_key]] = counts.get(row[env_key], 0) + 1
        summary[env_key] = counts

    metadata = {
        "env_a": env_data["A"].get("metadata", {}),
        "env_b": env_data["B"].get("metadata", {}),
        "env_c": env_data["C"].get("metadata", {}),
        "catalog": str(args.catalog) if args.catalog else None,
    }

    result = {
        "schema_version": 1,
        "metadata": metadata,
        "summary": summary,
        "rows": rows,
    }

    if ref_linux_data is not None:
        metadata["ref_linux"] = ref_linux_data.get("metadata", {})
        av_summary = {}
        ref_counts = {}
        for row in rows:
            av_summary[row["aster_vs_linux"]] = av_summary.get(row["aster_vs_linux"], 0) + 1
            ref_counts[row["ref_linux"]] = ref_counts.get(row["ref_linux"], 0) + 1
        result["aster_vs_linux_summary"] = av_summary
        summary["ref_linux"] = ref_counts

    return result


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--catalog", type=Path)
    parser.add_argument("--env-a", type=Path, required=True)
    parser.add_argument("--env-b", type=Path, required=True)
    parser.add_argument("--env-c", type=Path, required=True)
    parser.add_argument("--ref-linux", type=Path, default=None,
                        help="optional Linux/QEMU reference results for the Asterinas-vs-Linux comparison")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--allow-missing", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    matrix = build_matrix(args)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as f:
        json.dump(matrix, f, indent=2, sort_keys=True)
        f.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
