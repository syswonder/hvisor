#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
#
# Run the Asterinas Linux-ABI comparison workflow. The harness self-check needs
# no VM; guest boots need the prerequisites described in
# docs/hvisor_zone0_boot.md (a built Asterinas OSDK kernel, a Linux 5.19 bzImage,
# and a QEMU >= 9.x with KVM). This script runs the parts that are self-contained
# and drives the boots when their inputs are present.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"
QEMU="${ABI_QEMU:-qemu-system-x86_64}"
ACCEL="${ABI_ACCEL:-kvm}"
KERNEL_DIR="${ABI_KERNEL_DIR:-$ROOT/_build}"

echo "=== Asterinas-on-hvisor ABI differential ==="

echo "--- [1] host self-check (tests + runner + smoke + lint) ---"
make verify

echo "--- [2] autorun initramfs images (env A / B / C / reference) ---"
for env in A B C L; do
    ABI_AUTORUN_ENV="$env" ABI_AUTORUN_TIMEOUT=90 \
        sh initramfs/build_initramfs.sh "$ROOT/_build/initramfs-$env.cpio.gz" >/dev/null
done
echo "    initramfs images written under _build/"

# Env A: Asterinas on QEMU (multiboot2 grub-rescue ISO). Needs an OSDK ISO at
# $KERNEL_DIR/aster-envA.iso (see docs/hvisor_zone0_boot.md) and a QEMU >= 9.x.
if [ -f "$KERNEL_DIR/aster-envA.iso" ]; then
    echo "--- [3] Env A: Asterinas on QEMU ---"
    python3 harness/boot_collect.py --mode iso --iso "$KERNEL_DIR/aster-envA.iso" \
        --qemu "$QEMU" --accel "$ACCEL" --env-id A \
        --out results/env_a/results.json \
        --label "Asterinas on QEMU + $ACCEL"
else
    echo "--- [3] Env A skipped: $KERNEL_DIR/aster-envA.iso not present (see docs) ---"
fi

# Reference: Linux 5.19 direct bzImage boot (same suite, same emulator).
if [ -f "$KERNEL_DIR/linux-5.19-obj/arch/x86/boot/bzImage" ]; then
    echo "--- [4] reference: Linux 5.19 on QEMU ---"
    python3 harness/boot_collect.py --mode linux \
        --kernel "$KERNEL_DIR/linux-5.19-obj/arch/x86/boot/bzImage" \
        --initrd "$ROOT/_build/initramfs-L.cpio.gz" \
        --append "console=ttyS0 rdinit=/init" \
        --qemu "$QEMU" --accel "$ACCEL" --env-id ref_linux \
        --out results/ref_linux_qemu/results.json \
        --label "Linux 5.19 on QEMU + $ACCEL"
else
    echo "--- [4] reference Linux skipped: bzImage not present (see docs) ---"
fi

echo "--- [5] Env C: Asterinas on hvisor zone0 ---"
echo "    Build the hvisor ISO and boot it under QEMU + KVM as documented in"
echo "    docs/hvisor_zone0_boot.md (the boot writes the framed result JSON to"
echo "    the serial console; decode it into results/env_c/results.json)."

echo "--- [6] report ---"
make report
echo "=== done: report/out/compatibility_report.md ==="
