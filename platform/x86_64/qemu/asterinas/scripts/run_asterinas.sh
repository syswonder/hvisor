#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Turnkey Asterinas-on-hvisor run: build hvisor with CONFIG_ASTER_GUEST=y,
# boot it under QEMU/KVM with the Asterinas GRUB entry selected, and capture the
# serial console (the probe self-test runs as PID 1 and prints to it).
#
# Prerequisite: the Asterinas image has been staged with build_asterinas.sh.
# Usage: run_asterinas.sh [console.log] [timeout_s] [smp]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HV_ROOT="$(cd "$HERE/../../../.." && pwd)"
IMG="$HV_ROOT/platform/x86_64/qemu/image"
GRUB_CFG="$IMG/iso/boot/grub/grub.cfg"
LOG="${1:-$HERE/artifacts/asterinas-console.log}"
SECS="${2:-90}"
SMP="${3:-8}"

if [ ! -s "$IMG/kernel/asterinas-vmlinux.bin" ]; then
    echo "Asterinas image not staged; run scripts/build_asterinas.sh first." >&2
    exit 1
fi

# Select the "Asterinas" GRUB entry so the freshly built ISO boots unattended.
# On exit we restore the committed default (0 = Linux) in the source grub.cfg so
# the working tree and the next plain `make` produce a Linux-default ISO; the ISO
# this run builds stays Asterinas-default (rerun this script, or `make`, to switch
# back) since it also contains the aster_guest hvisor build.
restore_grub_default() { sed -i 's/^set default=1/set default=0/' "$GRUB_CFG"; }
trap restore_grub_default EXIT
sed -i 's/^set default=0/set default=1/' "$GRUB_CFG"

make -C "$HV_ROOT" ARCH=x86_64 BOARD=qemu defconfig
sed -i 's/^# CONFIG_ASTER_GUEST is not set$/CONFIG_ASTER_GUEST=y/' "$HV_ROOT/.config"
# Deliberately not `make ... MODE=release all`: the "all" chain runs the
# "vscode" step, which re-invokes the Kconfig defconfig generator and would
# silently overwrite the CONFIG_ASTER_GUEST=y edit above before the actual
# build. Ask for the same non-vscode targets "all" would otherwise build.
make -C "$HV_ROOT" ARCH=x86_64 BOARD=qemu MODE=release \
    gen_cargo_config target/x86_64-unknown-none/release/hvisor.bin check-hv-mem-overlap

mkdir -p "$(dirname "$LOG")"
: > "$LOG"
echo "Booting Asterinas (smp=$SMP, ${SECS}s); console -> $LOG"
timeout --kill-after=10 "$SECS" qemu-system-x86_64 \
    -machine q35,kernel-irqchip=split \
    -cpu host,+x2apic,+invtsc,+vmx -accel kvm \
    -smp "$SMP" -m 4G \
    -bios /usr/share/ovmf/OVMF.fd \
    -display none -nodefaults -net none \
    -serial file:"$LOG" \
    -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48 \
    -device ioh3420,id=pcie.1,chassis=1 \
    -drive file="$IMG/virtdisk/hvisor.iso",format=raw,index=0,media=disk \
    -no-reboot || true

echo "--- probe self-test output ---"
clean_log="$(sed 's/\x1b\[[0-9;]*[mHJ]//g; s/\r//g' "$LOG")"
printf '%s\n' "$clean_log" | awk '/guest probe self-test/{f=1} f'
if printf '%s\n' "$clean_log" | grep -q 'guest probe self-test COMPLETE'; then
    echo "Asterinas guest probe self-test: PASS"
else
    echo "ERROR: Asterinas guest self-test did not complete (no self-test marker in $LOG); the guest did not boot far enough to run the probes." >&2
    exit 1
fi
