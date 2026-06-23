#!/usr/bin/env bash
# Host-side isolation checks that do not require KVM:
# zonelint positives and negatives, the VirtIO backend guard, the fuzzer dry-run,
# and the fault probes.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

pass=0 fail=0
ok()   { echo "PASS: $1"; pass=$((pass + 1)); }
bad()  { echo "FAIL: $1"; fail=$((fail + 1)); }

echo "=== zonelint: build ==="
# Prefer the Rust validator; fall back to the equivalent Python implementation
# where cargo is unavailable.
if command -v cargo >/dev/null 2>&1; then
    ( cd zonelint && cargo build --release >/dev/null )
    ZL="zonelint/target/release/zonelint"
else
    echo "cargo not found; using the Python zonelint"
    ZL="python3 zonelint/zonelint.py"
fi

echo "=== zonelint: positive configs (expect PASS) ==="
if $ZL --zone configs/zone1_victim.json --zone configs/zone2_attacker.json \
        --virtio configs/virtio_cfg.json; then ok "positive configs"; else bad "positive configs"; fi

echo "=== zonelint: overlap negative (expect FAIL) ==="
if $ZL --zone configs/zone1_victim.json --zone configs/negative/zone_overlap_bad.json; then
    bad "overlap negative was accepted"; else ok "overlap negative rejected"; fi

echo "=== zonelint: pci-incomplete negative (expect FAIL) ==="
if $ZL --zone configs/negative/pci_incomplete_bad.json; then
    bad "pci-incomplete negative was accepted"; else ok "pci-incomplete negative rejected"; fi

echo "=== virtio backend before/after demo (expect 0 unexpected outcomes) ==="
demo="$(mktemp -d)/backend_guard_demo"
cc -O2 -Wall -Wextra -std=c11 virtio-fuzzer/backend_guard_demo.c -o "$demo"
if "$demo"; then ok "backend guard demo"; else bad "backend guard demo"; fi

echo "=== virtio-fuzzer dry-run ==="
( cd virtio-fuzzer && cargo run --quiet --release -- --case all --dry-run >/dev/null ) \
    && ok "virtio-fuzzer dry-run" || bad "virtio-fuzzer dry-run"

echo "=== fault probe (unmapped IPA, expect trap) ==="
make -C faultinj all >/dev/null
if faultinj/build/test_unmapped_ipa 0x20000000; then ok "fault probe trapped"; else bad "fault probe"; fi

echo
echo "summary: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
