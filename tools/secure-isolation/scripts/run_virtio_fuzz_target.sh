#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MMIO_BASE="${MMIO_BASE:-0x5950f000}"
QUEUE="${QUEUE:-0}"
DEV_MEM="${DEV_MEM:-/dev/mem}"
POST_KICK_MS="${POST_KICK_MS:-250}"
CASE_LIST="${CASE_LIST:-circular_descriptor oversize_len oob_addr unaligned_addr corrupt_avail_idx}"
OUT_DIR="${OUT_DIR:-${TMPDIR:-/tmp}/hvisor-virtio-fuzz-$(date +%Y%m%d-%H%M%S)}"

mkdir -p "$OUT_DIR"

echo "virtio fuzz log dir: $OUT_DIR"
echo "mmio_base=$MMIO_BASE queue=$QUEUE dev_mem=$DEV_MEM post_kick_ms=$POST_KICK_MS" | tee "$OUT_DIR/environment.txt"
id | tee -a "$OUT_DIR/environment.txt"
uname -a | tee -a "$OUT_DIR/environment.txt"

if [ ! -r "$DEV_MEM" ] || [ ! -w "$DEV_MEM" ]; then
    echo "ERROR: $DEV_MEM must be readable and writable. Run inside the guest as root." >&2
    exit 1
fi

if [ ! -r /proc/self/pagemap ]; then
    echo "ERROR: /proc/self/pagemap is not readable. CAP_SYS_ADMIN may be required." >&2
    exit 1
fi

if command -v cargo >/dev/null 2>&1; then
    CARGO="$(command -v cargo)"
else
    echo "ERROR: cargo not found; build virtio-fuzzer in the guest first." >&2
    exit 127
fi

for case_name in $CASE_LIST; do
    log="$OUT_DIR/$case_name.log"
    echo "=== $case_name ===" | tee "$log"
    "$CARGO" run --manifest-path "$ROOT/virtio-fuzzer/Cargo.toml" --quiet -- \
        --mmio-base "$MMIO_BASE" \
        --queue "$QUEUE" \
        --dev-mem "$DEV_MEM" \
        --post-kick-ms "$POST_KICK_MS" \
        --case "$case_name" \
        --execute 2>&1 | tee -a "$log"
    sync
    sleep 1
done

echo "done: logs in $OUT_DIR"
