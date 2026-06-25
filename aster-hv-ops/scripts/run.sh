#!/bin/bash
# SPDX-License-Identifier: MulanPSL-2.0
# Boot the Asterinas root zone under QEMU/KVM and capture the serial console.
# hvisor is a hypervisor and never exits on its own, so a wall-clock timeout is
# enforced; a selftest initramfs powers the machine off when its checks finish.
#
# Usage:
#   run.sh [iso] [logfile] [timeout_secs]
# Environment:
#   KVM=0   fall back to TCG (functional only; timings are not representative)
set -u

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
HV_ROOT="$(cd -- "$HERE/../.." && pwd)"
ISO="${1:-$HV_ROOT/platform/x86_64/qemu/image/virtdisk/hvisor.iso}"
LOG="${2:-$HERE/../results/run.log}"
TMO="${3:-45}"

[ -f "$ISO" ] || { echo "run.sh: ISO not found: $ISO" >&2; exit 1; }

if [ "${KVM:-1}" = 0 ]; then
    # -cpu host needs KVM; `max` already advertises x2apic/invtsc under TCG, so
    # no explicit +feature flags (which TCG would warn about) are needed. The VT-d
    # extended interrupt mode (eim=on) also requires KVM, so it is dropped here.
    accel="-accel tcg"
    cpu="-cpu max"
    iommu="-device intel-iommu,intremap=on,caching-mode=on,device-iotlb=on,aw-bits=48"
else
    accel="-accel kvm"
    cpu="-cpu host,+x2apic,+invtsc,+vmx"
    iommu="-device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48"
fi

: > "$LOG"
# The ISO is the boot disk (OVMF loads GRUB from it). timeout's exit status is
# preserved so a QEMU launch failure is not masked; 124 means the wall-clock
# limit was reached, which is the expected outcome for a running hypervisor.
timeout --foreground "$TMO" qemu-system-x86_64 \
    -machine q35,kernel-irqchip=split \
    $cpu $accel \
    -smp 4 -m 4G \
    -bios /usr/share/ovmf/OVMF.fd \
    -nographic -serial "file:$LOG" -nodefaults \
    $iommu \
    -device ioh3420,id=pcie.1,chassis=1 \
    -drive file="$ISO",format=raw,index=0,media=disk
rc=$?

# Clean the capture: drop VT100 sequences and stray control bytes from concurrent
# CPU output, strip trailing whitespace, and redact the host build directory from
# the Asterinas component paths it bakes in (keep the source-relative remainder).
# Done in Python (already required) so the only scripting dependency is python3.
python3 - "$LOG" <<'PY'
import re, sys
path = sys.argv[1]
with open(path, "r", encoding="utf-8", errors="replace") as f:
    s = f.read()
s = re.sub(r"\x1b\[[0-9;=?]*[A-Za-z]", "", s)      # VT100 escape sequences
s = s.replace("\r", "")
s = re.sub(r"[^\t\n\x20-\x7e]", "", s)              # stray control bytes
s = re.sub(r"/[^\" \n\r]*/asterinas/", "asterinas/", s)  # redact host build dir (per line)
s = re.sub(r"[ \t]+$", "", s, flags=re.M)           # trailing whitespace
with open(path, "w", encoding="utf-8") as f:
    f.write(s)
PY
sanitize_rc=$?
if [ "$sanitize_rc" -ne 0 ]; then
    echo "run.sh: log sanitize failed (status $sanitize_rc); not reporting success" >&2
    exit "$sanitize_rc"
fi
echo "run.sh: console captured -> $LOG"

# 124 means the wall-clock limit was reached, the expected outcome for a running
# hypervisor; any other non-zero status is a genuine QEMU failure and propagates.
if [ "$rc" -ne 0 ] && [ "$rc" -ne 124 ]; then
    echo "run.sh: qemu exited with status $rc" >&2
    exit "$rc"
fi
