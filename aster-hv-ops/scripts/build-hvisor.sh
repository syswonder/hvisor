#!/bin/bash
# SPDX-License-Identifier: MulanPSL-2.0
# Build the hvisor x86_64 binary with the Asterinas root-zone profile and
# assemble the bootable ISO. The Asterinas bzImage and initramfs must already be
# staged into the qemu image tree (see build-asterinas.sh / mkinitramfs.sh, or
# the hv-asterctl `build` command which does both).
#
# Usage:
#   build-hvisor.sh
#
# Board options come from the qemu defconfig (pci, ecam_pcie,
# no_pcie_bar_realloc, uart16550a, intel_vtd); this script additionally flips
# on the opt-in CONFIG_ASTER_GUEST toggle by editing the generated .config,
# since the old FEATURES=... make override was removed upstream.
set -eu

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
HV_ROOT="$(cd -- "$HERE/../.." && pwd)"

cd "$HV_ROOT"
ISO="platform/x86_64/qemu/image/virtdisk/hvisor.iso"
# Drop any previous ISO so a skipped build step (e.g. xorriso not installed) is
# caught by the existence check below instead of passing on a stale image.
rm -f "$ISO"

make ARCH=x86_64 BOARD=qemu defconfig
sed -i 's/^# CONFIG_ASTER_GUEST is not set$/CONFIG_ASTER_GUEST=y/' .config

echo "[hvisor] CONFIG_ASTER_GUEST=y"
echo "[hvisor] toolchain: $(rustc --version)"
make ARCH=x86_64 BOARD=qemu all

[ -f "$ISO" ] || {
    echo "[hvisor] ISO not produced at $ISO (is xorriso/grub-mkrescue installed?)" >&2
    exit 1
}
echo "[hvisor] done -> $ISO ($(du -h "$ISO" | cut -f1))"
