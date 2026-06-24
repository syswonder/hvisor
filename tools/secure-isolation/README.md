# secure-isolation

Utilities for checking hvisor isolation behavior when Asterinas runs as a
non-root x86_64 guest.

The tools cover static zone-config checks, malformed VirtIO descriptor inputs,
guest-side fault probes, and local launch helpers for the QEMU/KVM x86_64 board.
They are intended to sit next to the hvisor source changes that contain
unhandled EPT faults, propagate MMIO emulation failures, validate zone memory
ranges, and support Asterinas boot through the Linux/x86 boot protocol.

## Contents

- `zonelint/` - Rust and Python validators for zone and VirtIO JSON.
- `virtio-fuzzer/` - split-virtqueue input generator plus a host-side backend
  buffer-boundary check.
- `faultinj/` - guest probes for unmapped IPA, cross-zone memory, unauthorized
  MMIO, and noisy-neighbor latency.
- `configs/` - sample zone and VirtIO JSON, including negative fixtures.
- `scripts/` - local build, boot, fuzz, and probe helpers.
- `hardening-patches/harden_virtio_backend.patch` - hvisor-tool VirtIO backend
  bounds checks for descriptor walks and descriptor buffers.
- `threat-model/THREAT_MODEL.md` - isolation assumptions and hardening notes.

## Host Checks

The host-side checks do not require KVM:

```bash
cd tools/secure-isolation
scripts/reproduce.sh
```

The script builds `zonelint`, checks the positive and negative fixtures, runs the
VirtIO backend guard, exercises the fuzzer in dry-run mode, and builds the fault
probe binaries.

Individual checks can also be run directly:

```bash
( cd zonelint && cargo build --release )
./zonelint/target/release/zonelint \
    --zone configs/zone1_victim.json \
    --zone configs/zone2_attacker.json \
    --virtio configs/virtio_cfg.json

./zonelint/target/release/zonelint \
    --zone configs/zone1_victim.json \
    --zone configs/negative/zone_overlap_bad.json

cc -O2 -Wall -Wextra -std=c11 \
    virtio-fuzzer/backend_guard_demo.c -o /tmp/backend_guard_demo
/tmp/backend_guard_demo

( cd virtio-fuzzer && cargo run --release -- --case all --dry-run )
make -C faultinj all
faultinj/build/test_unmapped_ipa 0x20000000
```

## hvisor-tool Backend Patch

Apply the VirtIO backend patch from an hvisor-tool source tree:

```bash
git -C <hvisor-tool> apply --check \
    tools/secure-isolation/hardening-patches/harden_virtio_backend.patch
git -C <hvisor-tool> apply \
    tools/secure-isolation/hardening-patches/harden_virtio_backend.patch
make -C <hvisor-tool>/tools hvisor ARCH=x86_64 LOG=LOG_INFO
```

## QEMU/KVM Boot Helper

Build hvisor for the x86_64 QEMU board, provide a zone0 Linux kernel/rootfs in
the board image directory, and boot with:

```bash
scripts/run_hvisor_qemu.sh <hvisor.iso> <zone0_rootfs.img> serial.log
```

The helper requires `/dev/kvm`; hvisor is a VMX hypervisor and does not boot
under TCG.

## Guest Checks

After zone0 has started the non-root zones, run the C probes from the intended
guest and inspect the hvisor logs for `isolation-fault`, `isolation-pio`, and
`isolation-msr` records:

```bash
scripts/run_isolation_suite.sh
scripts/run_virtio_fuzz_target.sh
```

The VirtIO execute-mode fuzzer needs access to the owning guest's MMIO window,
`/dev/mem`, and `/proc/self/pagemap`.
