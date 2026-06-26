#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Validate that tests/catalog.json matches executable test sources."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    catalog_path = ROOT / "tests" / "catalog.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))["tests"]
    catalog_name_list = [row["name"] for row in catalog]
    catalog_names = set(catalog_name_list)

    # The catalog must not list a name twice: a duplicate would silently mask a
    # diverging row and corrupt the differential matrix keyed by name.
    if len(catalog_name_list) != len(catalog_names):
        seen = set()
        dups = sorted({n for n in catalog_name_list if n in seen or seen.add(n)})
        print(f"duplicate_catalog_names={dups}")
        return 1

    actual_list = []
    for path in (ROOT / "tests").glob("**/*.c"):
        if path.name.startswith("t_"):
            actual_list.append(path.stem)
    for base in [ROOT / "tests" / "l5_observability", ROOT / "tests" / "real_apps"]:
        for path in base.glob("*.sh"):
            actual_list.append(path.stem)
    actual = set(actual_list)

    # Two test files collapsing to the same stem would each map to one catalog
    # row and quietly drop a case from coverage; reject that too.
    if len(actual_list) != len(actual):
        seen = set()
        dups = sorted({n for n in actual_list if n in seen or seen.add(n)})
        print(f"duplicate_test_file_names={dups}")
        return 1

    missing = sorted(actual - catalog_names)
    stale = sorted(catalog_names - actual)
    if missing or stale:
        print(f"missing_in_catalog={missing}")
        print(f"catalog_without_file={stale}")
        return 1
    print(f"catalog ok: {len(actual)} tests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
