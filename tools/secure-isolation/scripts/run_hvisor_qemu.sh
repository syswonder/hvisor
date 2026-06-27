#!/usr/bin/env bash
# Boot the hvisor x86_64 GRUB image under QEMU + KVM (nested VT-x) and capture
# the serial console.
# It refuses to run without /dev/kvm: hvisor is a VMX hypervisor and cannot boot
# under TCG.
#
# Usage: run_hvisor_qemu.sh <hvisor.iso> <zone0_rootfs.img> [serial-log]
set -euo pipefail

ISO="${1:?usage: run_hvisor_qemu.sh <hvisor.iso> <zone0_rootfs.img> [serial-log]}"
ROOTFS="${2:?usage: run_hvisor_qemu.sh <hvisor.iso> <zone0_rootfs.img> [serial-log]}"
SERIAL="${3:-hvisor-serial.log}"
SMP="${SMP:-6}"
MEM="${MEM:-4G}"
TIMEOUT="${TIMEOUT:-120}"
OVMF="${OVMF:-/usr/share/ovmf/OVMF.fd}"

if [ ! -e /dev/kvm ]; then
    echo "ERROR: /dev/kvm missing; hvisor is a VMX hypervisor and cannot boot under TCG." >&2
    exit 1
fi

echo "booting $ISO (rootfs=$ROOTFS, smp=$SMP, mem=$MEM, timeout=${TIMEOUT}s)"
echo "serial -> $SERIAL"

set +e
timeout "$TIMEOUT" qemu-system-x86_64 \
    -machine q35,kernel-irqchip=split \
    -cpu host,+x2apic,+invtsc,+vmx -accel kvm \
    -smp "$SMP" -m "$MEM" \
    -bios "$OVMF" \
    -display none -serial "file:$SERIAL" \
    -nodefaults -net nic -net user \
    -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48 \
    -device ioh3420,id=pcie.1,chassis=1 \
    -drive if=none,file="$ROOTFS",id=X10008000,format=raw \
    -device virtio-blk-pci,bus=pcie.1,drive=X10008000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on \
    -drive file="$ISO",format=raw,index=0,media=disk \
    </dev/null
status=$?
set -e

echo "=== serial tail ==="
if [ -f "$SERIAL" ]; then
    tail -n 20 "$SERIAL" | sed 's/\x1b\[[0-9;]*[a-zA-Z]//g'
else
    echo "serial log was not created"
fi

exit "$status"
