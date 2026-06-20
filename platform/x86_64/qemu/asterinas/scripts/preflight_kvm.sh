#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Preflight check for the Asterinas-on-hvisor runtime. hvisor's x86 port uses
# VT-x/VMX and the qemu target runs it under QEMU/KVM, so the host must expose
# hardware virtualization.
#
# Exit 0 = KVM usable. Exit 1 = KVM unavailable.
set -u

ok=1
say() { printf '%s\n' "$*"; }
hdr() { printf '\n== %s ==\n' "$*"; }

hdr "CPU virtualization flag (vmx/svm)"
vmx_count=$(grep -cE '(^|[[:space:]])(vmx|svm)([[:space:]]|$)' /proc/cpuinfo 2>/dev/null || true)
vmx_count=${vmx_count:-0}
if [ "$vmx_count" -gt 0 ] 2>/dev/null; then
  say "  OK: $vmx_count logical CPUs expose vmx/svm"
else
  say "  MISSING: no vmx/svm in /proc/cpuinfo"
  if command -v dmesg >/dev/null 2>&1; then
    bios=$(dmesg 2>/dev/null | grep -i 'kvm: disabled by bios' | tail -1 || true)
    [ -n "$bios" ] && say "  -> $bios  (enable 'Intel VT-x'/'AMD-V' in BIOS/UEFI and reboot)"
  fi
  ok=0
fi

hdr "/dev/kvm device node"
if [ -e /dev/kvm ]; then
  say "  OK: /dev/kvm present"
  [ -r /dev/kvm ] && [ -w /dev/kvm ] && say "  OK: /dev/kvm is readable+writable by $(id -un)" \
    || say "  WARN: /dev/kvm not r/w for this user (add user to the kvm group)"
else
  say "  MISSING: /dev/kvm does not exist (load kvm_intel/kvm_amd, or enable VT-x in BIOS)"
  ok=0
fi

hdr "Nested virtualization (needed only if the host is itself a VM)"
for m in kvm_intel kvm_amd; do
  f="/sys/module/$m/parameters/nested"
  if [ -e "$f" ]; then
    say "  $m nested = $(cat "$f" 2>/dev/null)"
  fi
done
say "  (nested=Y required only when this host is a guest; bare-metal hosts can ignore)"

hdr "QEMU with KVM accelerator"
qemu_bin="$(command -v qemu-system-x86_64 || true)"
if [ -n "$qemu_bin" ]; then
  if "$qemu_bin" -accel help 2>/dev/null | grep -qx kvm; then
    say "  OK: $qemu_bin supports accel=kvm"
  else
    say "  WARN: $qemu_bin does not advertise kvm accel"
  fi
else
  say "  MISSING: qemu-system-x86_64 not in PATH (install qemu-system-x86)"
  ok=0
fi

hdr "Verdict"
if [ "$ok" -eq 1 ]; then
  say "  OK: this host can run hvisor + Asterinas under QEMU/KVM."
  exit 0
else
  say "  ERROR: hardware virtualization is unavailable on this host."
  say "  hvisor/KVM cannot run here; only TCG functional diagnostics are possible,"
  say "  and TCG numbers are NOT valid for performance/SMP/isolation conclusions."
  say "  Run this on a KVM-capable x86 host."
  exit 1
fi
