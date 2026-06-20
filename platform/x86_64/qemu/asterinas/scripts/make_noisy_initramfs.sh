#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Build the noisy-neighbour initramfs for zone2. PID 1 is noisy_init, which
# saturates the zone's pCPUs and shared LLC/DRAM bandwidth. mem_burn is also
# included for sustained allocator/TLB churn.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/artifacts/noisy-initramfs/initramfs.cpio.gz}"
WORK="$ROOT/artifacts/noisy-initramfs/root"
GUEST_BIN="$ROOT/build/guest-bin"

make -C "$ROOT" guest >/dev/null

rm -rf "$WORK"
mkdir -p "$WORK"/{dev,proc,sys,tmp,bin}

install -m 0755 "$GUEST_BIN/mem_burn" "$WORK/bin/mem_burn"
install -m 0755 "$GUEST_BIN/noisy_init" "$WORK/init"

mkdir -p "$(dirname "$OUT")"
(
  cd "$WORK"
  find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$OUT"
)

echo "noisy initramfs: $OUT"
echo "  /init      -> noisy_init (cpu + LLC + DRAM interference on all zone cpus)"
echo "  /bin/mem_burn"
ls -l "$OUT"
