#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
"""Split a Linux boot protocol bzImage into setup and protected-mode payload."""

from __future__ import annotations

import argparse
import json
import struct
from pathlib import Path


def u8(blob: bytes, off: int) -> int:
    return struct.unpack_from("B", blob, off)[0]


def u16(blob: bytes, off: int) -> int:
    return struct.unpack_from("<H", blob, off)[0]


def u32(blob: bytes, off: int) -> int:
    return struct.unpack_from("<I", blob, off)[0]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bzimage", type=Path)
    parser.add_argument("--setup-out", type=Path, default=Path("artifacts/asterinas-setup.bin"))
    parser.add_argument("--kernel-out", type=Path, default=Path("artifacts/asterinas-vmlinux.bin"))
    parser.add_argument("--metadata-out", type=Path, default=Path("artifacts/asterinas-bzimage-meta.json"))
    parser.add_argument("--min-proto", default="0x0204")
    parser.add_argument("--expected-code32", default="0x100000")
    args = parser.parse_args()

    blob = args.bzimage.read_bytes()
    if len(blob) < 0x300:
        raise SystemExit(f"{args.bzimage} is too small to be a bzImage")

    setup_sects = u8(blob, 0x1F1) or 4
    split = (setup_sects + 1) * 512
    proto = u16(blob, 0x206)
    code32 = u32(blob, 0x214)
    min_proto = int(args.min_proto, 0)
    expected_code32 = int(args.expected_code32, 0)

    if split >= len(blob):
        raise SystemExit(f"invalid setup split offset {split}, file size {len(blob)}")
    if proto < min_proto:
        raise SystemExit(f"boot protocol 0x{proto:04x} < required 0x{min_proto:04x}")
    if code32 != expected_code32:
        raise SystemExit(f"code32_start 0x{code32:08x} != expected 0x{expected_code32:08x}")

    args.setup_out.parent.mkdir(parents=True, exist_ok=True)
    args.kernel_out.parent.mkdir(parents=True, exist_ok=True)
    args.metadata_out.parent.mkdir(parents=True, exist_ok=True)
    args.setup_out.write_bytes(blob[:split])
    args.kernel_out.write_bytes(blob[split:])

    metadata = {
        "source": str(args.bzimage),
        "source_size": len(blob),
        "setup_sects": setup_sects,
        "split_offset": split,
        "boot_proto_version": f"0x{proto:04x}",
        "code32_start": f"0x{code32:08x}",
        "setup_out": str(args.setup_out),
        "kernel_out": str(args.kernel_out),
    }
    args.metadata_out.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(metadata, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
