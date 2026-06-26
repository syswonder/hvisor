#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Syntax-check Python files without writing __pycache__ artifacts."""

from __future__ import annotations

import ast
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def iter_python(paths: list[str]):
    """Yield .py files for each argument.

    A directory expands to its top-level .py files; a .py file yields itself.
    An argument that resolves to neither (e.g. a typo or deleted path) raises
    FileNotFoundError so the gate fails loudly instead of silently returning
    "ok" on a path it never checked.
    """
    for raw in paths:
        path = (ROOT / raw).resolve()
        if path.is_file() and path.suffix == ".py":
            yield path
        elif path.is_dir():
            yield from sorted(path.glob("*.py"))
        else:
            raise FileNotFoundError(raw)


def main(argv: list[str]) -> int:
    errors = []
    try:
        targets = list(iter_python(argv))
    except FileNotFoundError as exc:
        print(f"not a .py file or directory: {exc}")
        return 1
    for path in targets:
        try:
            ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        except SyntaxError as exc:
            errors.append(f"{path.relative_to(ROOT)}:{exc.lineno}: {exc.msg}")
    if errors:
        for error in errors:
            print(error)
        return 1
    print("python syntax ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
