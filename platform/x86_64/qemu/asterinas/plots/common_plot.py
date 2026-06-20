# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
"""Shared plotting helpers."""

from __future__ import annotations

from pathlib import Path


def read_samples(path: Path) -> list[float]:
    values: list[float] = []
    with path.open("r", encoding="utf-8") as stream:
        for line in stream:
            text = line.strip()
            if not text or text.startswith("#"):
                continue
            try:
                values.append(float(text))
            except ValueError:
                continue
    if not values:
        raise ValueError(f"{path} has no numeric samples")
    return values


def label_from_path(path: Path) -> str:
    return path.stem.replace("_", " ")
