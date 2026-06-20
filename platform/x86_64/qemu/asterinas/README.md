<!-- SPDX-License-Identifier: MulanPSL-2.0 -->
<!-- Copyright (c) 2026 hvisor contributors -->
# Asterinas guest on hvisor (x86_64/qemu)

This directory adapts the [Asterinas](https://github.com/asterinas/asterinas)
kernel to run as an x86_64 guest under hvisor, and provides a small SMP/RT probe
suite to verify CPU, interrupt and timer behaviour inside the guest.

Asterinas is booted through the same Linux/x86 boot protocol hvisor already uses
for its Linux root zone: the kernel is built with OSDK as a `linux-legacy32`
bzImage, split into a setup blob and a kernel payload, and entered at
`code32_start` (`0x100000`) with a `boot_params`/zeropage prepared by hvisor.
The enabling hypervisor changes live in `src/arch/x86_64` and
`platform/x86_64/qemu/board.rs`; this directory holds the guest-side image
pipeline, configurations, probes and helper scripts.

## Layout

```
asterinas/
  Makefile             build the C probes (host + static guest payload)
  probes/              smp_probe ipi_pingpong timer_jitter ctxsw  (+ common.h)
  noisy/               interference workloads for multi-zone load checks
  rootfs/              initramfs PID-1 sources (probe self-test, noisy, minimal)
  pipeline/stats.py    percentile summariser for raw probe streams
  plots/               matplotlib renderers for probe output
  configs/             Asterinas zone JSON (1/2/4-core) + noisy zone + virtio
  scripts/             image build, initramfs, config check, run/matrix drivers
```

## Quick start (KVM-capable x86_64 host)

```bash
# 0. gate: VT-x + /dev/kvm + nested virtualization
platform/x86_64/qemu/asterinas/scripts/preflight_kvm.sh

# 1. build the Asterinas image (needs an Asterinas + linux_vdso checkout)
export ASTERINAS_DIR=/path/to/asterinas         # tested at f0958799
export VDSO_LIBRARY_DIR=/path/to/linux_vdso     # asterinas/linux_vdso
platform/x86_64/qemu/asterinas/scripts/build_asterinas.sh

# 2. build hvisor with the Asterinas root zone and boot it under QEMU/KVM
platform/x86_64/qemu/asterinas/scripts/run_asterinas.sh
```

`run_asterinas.sh` builds hvisor with `FEATURES="... aster_guest"`, selects the
`Asterinas` GRUB entry and boots it headless. The probe self-test runs as PID 1
and prints the SMP topology, context-switch rate, timer jitter and cross-core
round-trip summaries to the serial console.

The default `make ARCH=x86_64 BOARD=qemu` build is unchanged and still boots the
Linux root zone from its virtio-blk disk.

## Compliance

hvisor is licensed under Mulan PSL v2; Asterinas under MPL-2.0. New files in this
directory carry an `SPDX-License-Identifier: MulanPSL-2.0` header. No Asterinas
sources are vendored here; the image build uses an external checkout selected
with `ASTERINAS_DIR`.
