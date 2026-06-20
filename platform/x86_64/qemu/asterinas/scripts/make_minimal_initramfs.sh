#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/artifacts/minimal-initramfs/initramfs.cpio.gz}"
WORK="$ROOT/artifacts/minimal-initramfs/root"
CC="${CC:-${GUEST_CC:-musl-gcc}}"

mkdir -p "$(dirname "$OUT")"
rm -rf "$WORK"
mkdir -p "$WORK"/{dev,proc,sys,tmp,bin}

"$CC" -O2 -static "$ROOT/rootfs/minimal_init.c" -o "$WORK/init"
chmod +x "$WORK/init"

(
  cd "$WORK"
  find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$OUT"
)

echo "$OUT"
