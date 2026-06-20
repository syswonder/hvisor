#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Dry-run fallback for the Rust VirtIO fuzzer."""

from __future__ import annotations

import argparse
from dataclasses import dataclass


QUEUE_SIZE = 8
DEFAULT_MMIO_BASE = 0x5950F000
DEFAULT_QUEUE = 0
VRING_DESC_F_NEXT = 1
VRING_DESC_F_WRITE = 2


@dataclass
class Descriptor:
    addr: int
    length: int
    flags: int
    next: int


@dataclass
class Ring:
    desc0: Descriptor
    avail_flags: int = 0
    avail_idx: int = 1
    avail_ring0: int = 0


def default_ring() -> Ring:
    return Ring(desc0=Descriptor(addr=0x100000, length=512, flags=0, next=0))


def mutate(case: str) -> Ring:
    ring = default_ring()
    if case == "circular_descriptor":
        ring.desc0.flags = VRING_DESC_F_NEXT
        ring.desc0.next = 0
    elif case == "oversize_len":
        ring.desc0.length = 0xFFFFFFFF
        ring.desc0.flags = VRING_DESC_F_WRITE
    elif case == "oob_addr":
        ring.desc0.addr = 0xDEADBEEF
        ring.desc0.length = 4096
    elif case == "unaligned_addr":
        ring.desc0.addr = 0x100001
        ring.desc0.length = 128
    elif case == "corrupt_avail_idx":
        ring.avail_idx = 0xFFFF
    else:
        raise ValueError(f"unknown case: {case}")
    return ring


def describe(case: str) -> str:
    return {
        "circular_descriptor": "desc[0] points back to itself with VRING_DESC_F_NEXT",
        "oversize_len": "desc[0].len is 0xffff_ffff",
        "oob_addr": "desc[0].addr is outside the declared zone memory",
        "unaligned_addr": "desc[0].addr is deliberately unaligned",
        "corrupt_avail_idx": "avail.idx jumps far beyond the queue size",
    }[case]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mmio-base", default=hex(DEFAULT_MMIO_BASE))
    parser.add_argument("--queue", type=int, default=DEFAULT_QUEUE)
    parser.add_argument("--case", action="append")
    parser.add_argument("--dry-run", action="store_true", default=True)
    args = parser.parse_args()

    cases = []
    for item in args.case or ["all"]:
        if item == "all":
            cases.extend(
                [
                    "circular_descriptor",
                    "oversize_len",
                    "oob_addr",
                    "unaligned_addr",
                    "corrupt_avail_idx",
                ]
            )
        else:
            cases.append(item)

    print(
        f"virtio-fuzzer-fallback: mmio_base={args.mmio_base} "
        f"queue={args.queue} mode=dry-run"
    )
    for case in cases:
        ring = mutate(case)
        print()
        print(f"[{case}] {describe(case)}")
        print(
            "desc[0]: "
            f"addr=0x{ring.desc0.addr:x} len=0x{ring.desc0.length:x} "
            f"flags=0x{ring.desc0.flags:x} next={ring.desc0.next}"
        )
        print(
            f"avail: flags=0x{ring.avail_flags:x} "
            f"idx={ring.avail_idx} ring0={ring.avail_ring0}"
        )
        print(
            f"[{case}] dry-run: would write QueueSel={args.queue}, "
            f"QueueNum={QUEUE_SIZE}, QueueReady=1, QueueNotify={args.queue}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
