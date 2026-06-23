#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Build a self-testing guest initramfs for the Asterinas zone1 image. The
# initramfs embeds the statically-linked SMP/RT probes under /bin and uses the
# probe_selftest launcher as PID 1 (/init), so booting the image immediately
# runs the SMP/timer/IPI/ctxsw smoke checks and prints probe summaries to the
# console. The same probes can be driven manually for longer runs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/artifacts/guest-initramfs/initramfs.cpio.gz}"
WORK="$ROOT/artifacts/guest-initramfs/root"
GUEST_BIN="$ROOT/build/guest-bin"

# Ensure the statically-linked guest payload exists.
make -C "$ROOT" guest >/dev/null

rm -rf "$WORK"
mkdir -p "$WORK"/{dev,proc,sys,tmp,bin,benchmark}

for b in smp_probe ipi_pingpong timer_jitter ctxsw mem_burn probe_selftest; do
  install -m 0755 "$GUEST_BIN/$b" "$WORK/bin/$b"
done

# probe_selftest doubles as PID 1 so an unattended boot self-tests.
install -m 0755 "$GUEST_BIN/probe_selftest" "$WORK/init"

mkdir -p "$(dirname "$OUT")"
# Resolve OUT to an absolute path so the cpio redirect below still targets the
# intended location after the subshell changes into "$WORK".
OUT="$(cd "$(dirname "$OUT")" && pwd)/$(basename "$OUT")"
(
  cd "$WORK"
  find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$OUT"
)

echo "guest initramfs: $OUT"
echo "  /init           -> probe_selftest (auto self-test on boot)"
echo "  /bin/{smp_probe,ipi_pingpong,timer_jitter,ctxsw,mem_burn,probe_selftest}"
ls -l "$OUT"
