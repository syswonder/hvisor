#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Turnkey Asterinas-on-hvisor run: build hvisor with the aster_guest feature,
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
FEATURES="pci ecam_pcie no_pcie_bar_realloc uart16550a intel_vtd aster_guest"

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

make -C "$HV_ROOT" ARCH=x86_64 BOARD=qemu FEATURES="$FEATURES"

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
sed 's/\x1b\[[0-9;]*[mHJ]//g; s/\r//g' "$LOG" | awk '/guest probe self-test/{f=1} f'
