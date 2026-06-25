#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Print the Linux/x86 boot-header fields of a bzImage and the GPA at which it
must be loaded so its protected-mode entry lands at the board's 1 MiB entry.

The hvisor qemu board loads the whole bzImage as one module: its setup header is
placed at `setup_load_gpa` and the protected-mode kernel directly follows. For
the kernel to start at 0x100000, the image is loaded at 0x100000 - setup_size.
"""
import struct
import sys

KERNEL_ENTRY_GPA = 0x100000
ROOT_ZONE_KERNEL_HPA = 0x5000000  # board ROOT_ZONE_KERNEL_ADDR


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: bzimage-info.py <bzImage>", file=sys.stderr)
        return 2
    with open(sys.argv[1], "rb") as f:
        data = f.read()
    if len(data) < 0x218:
        print(f"{sys.argv[1]}: too short to be a bzImage ({len(data)} bytes)", file=sys.stderr)
        return 2
    setup_sects = data[0x1F1] or 4
    setup_size = (setup_sects + 1) * 512
    boot_flag = struct.unpack("<H", data[0x1FE:0x200])[0]
    header = data[0x202:0x206]
    proto = struct.unpack("<H", data[0x206:0x208])[0]
    code32 = struct.unpack("<I", data[0x214:0x218])[0]

    ok = boot_flag == 0xAA55 and header == b"HdrS" and proto >= 0x0204
    load_gpa = KERNEL_ENTRY_GPA - setup_size
    load_hpa = ROOT_ZONE_KERNEL_HPA + load_gpa

    print(f"file              {sys.argv[1]}")
    print(f"size              {len(data):#x} ({len(data)} bytes)")
    print(f"setup_sects       {setup_sects}")
    print(f"setup_size        {setup_size:#x}")
    print(f"boot flag (AA55)  {boot_flag:#06x} {'ok' if boot_flag == 0xAA55 else 'BAD'}")
    print(f"header magic      {header!r} {'ok' if header == b'HdrS' else 'BAD'}")
    print(f"boot protocol     {proto:#06x} {'ok' if proto >= 0x0204 else 'TOO OLD'}")
    print(f"code32_start      {code32:#x}")
    print(f"setup_load_gpa    {load_gpa:#x}   (board ROOT_ZONE_SETUP_ADDR)")
    print(f"grub module hpa   {load_hpa:#x}   (Asterinas menuentry load address)")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
