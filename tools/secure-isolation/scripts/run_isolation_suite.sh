#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FAULT_DIR="$ROOT/faultinj"

if [ "${RUN_GUEST_PROBES:-0}" != "1" ]; then
    cat <<'EOF'
This script is intended to run inside a started non-root guest. Set
RUN_GUEST_PROBES=1 after the zones are running to execute the address probes.
Host execution only proves host signal behavior and is not an hvisor isolation
result.
EOF
    exit 2
fi

echo "=== build guest-side probes ==="
make -C "$FAULT_DIR" all

echo
echo "=== space-isolation probes ==="
for t in test_unmapped_ipa test_cross_zone_mem test_unauthorized_mmio; do
    echo "[$t]"
    "$FAULT_DIR/build/$t" || true
done

echo
echo "=== latency probe ==="
"$FAULT_DIR/build/test_noisy_neighbor" || true

echo
echo "=== virtio fuzz dry-run ==="
if command -v cargo >/dev/null 2>&1; then
    ( cd "$ROOT/virtio-fuzzer" && cargo run --quiet -- --case all --dry-run )
elif command -v python3 >/dev/null 2>&1; then
    python3 "$ROOT/virtio-fuzzer/fuzz.py" --case all --dry-run
else
    echo "SKIP: neither cargo nor python3 found."
fi

cat <<'EOF'

Target-platform checks (need a KVM host):
1. Build hvisor and the matching hvisor-tool control plane for the rootfs.
2. Boot the zones: scripts/run_hvisor_qemu.sh <hvisor.iso> <zone0_rootfs.img>;
   then start zone1/zone2 from zone0 with configs/*.json.
3. Run these probes inside the intended guest zones.
4. Collect the hvisor 'isolation-fault' / 'isolation-pio' / 'isolation-msr'
   records added by the trap.rs and mmio.rs hardening.
EOF
