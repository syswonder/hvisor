#!/usr/bin/env bash
# Host-side checks that do not require KVM. Guest isolation results still require
# booting hvisor and running the probes in the target non-root zones.
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
    ( cd zonelint && cargo test --release )
    ZL="zonelint/target/release/zonelint"
    ok "zonelint unit tests"
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

echo "=== zonelint: virtio overlap negative (expect FAIL) ==="
if $ZL --zone configs/zone1_victim.json --zone configs/zone2_attacker.json \
        --virtio configs/negative/virtio_overlap_bad.json; then
    bad "virtio overlap negative was accepted"; else ok "virtio overlap negative rejected"; fi

echo "=== virtio-fuzzer dry-run ==="
( cd virtio-fuzzer && cargo test --quiet --release ) \
    && ok "virtio-fuzzer unit tests" || bad "virtio-fuzzer unit tests"
( cd virtio-fuzzer && cargo run --quiet --release -- --case all --dry-run >/dev/null ) \
    && ok "virtio-fuzzer dry-run" || bad "virtio-fuzzer dry-run"

echo "=== guest fault probes: build only ==="
make -C faultinj all >/dev/null
ok "fault probes build"

echo
echo "summary: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
