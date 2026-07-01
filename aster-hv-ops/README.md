<!-- SPDX-License-Identifier: MulanPSL-2.0 -->
# aster-hv-ops

Build and run tooling for [Asterinas](https://github.com/asterinas/asterinas) as
an hvisor **root zone** on x86_64 over the Linux/x86 boot protocol.

The hypervisor changes live in the hvisor tree. This directory builds the guest
bzImage, creates a small BusyBox initramfs, assembles the qemu ISO with the
`aster_guest` feature, and provides run, verify, and benchmark helpers.
Generated images and logs stay under ignored local output directories.

## Layout

```
aster-hv-ops/
├── hv-asterctl            control plane: build / run / verify / bench
├── scripts/
│   ├── build-asterinas.sh build the Asterinas bzImage with OSDK (linux legacy boot)
│   ├── mkinitramfs.sh     build the busybox initramfs (interactive | selftest)
│   ├── build-hvisor.sh    build hvisor (aster_guest) and assemble the ISO
│   ├── run.sh             boot under QEMU/KVM, capture the serial console
│   ├── bench-coldstart.py cold-start latency over N runs
│   └── bzimage-info.py    print bzImage boot-header fields + derived load address
```

## Prerequisites

- A host with `/dev/kvm` and Intel VT-x (nested virtualization if itself a guest).
- `qemu-system-x86_64` (>= 6.2) and OVMF firmware discoverable from the QEMU
  install prefix, or set with `OVMF`.
- `grub-mkrescue` + `xorriso`, a static `busybox`, `git`, `python3`.
- The Rust toolchains hvisor and Asterinas pin (`rust-toolchain.toml` in each
  tree); the Asterinas build installs its in-tree `cargo-osdk` under
  `aster-hv-ops/build/osdk` and fetches the pinned `linux_vdso` checkout unless
  `VDSO_LIBRARY_DIR` is already set.

## Quick start

```sh
# from the hvisor repo root
cd aster-hv-ops

./hv-asterctl build      # Asterinas bzImage + initramfs + hvisor ISO (aster_guest)
./hv-asterctl run        # boot the zone, print the captured console (Asterinas shell)
./hv-asterctl verify     # boot the selftest initramfs, assert PASS
./hv-asterctl bench      # cold-start latency, writes results/coldstart_bench.json
```

`build`, `verify`, and `bench` rebuild the guest image by default. Pass
`--reuse-guest` only when intentionally testing a previously built
`build/aster-kernel-osdk-bin`.

Without privileges you can still inspect a prebuilt bzImage:

```sh
scripts/bzimage-info.py build/aster-kernel-osdk-bin
```

Equivalently, hvisor's own Makefile drives the build and a headless run:

```sh
# from the hvisor repo root
make ARCH=x86_64 BOARD=qemu defconfig
sed -i 's/^# CONFIG_ASTER_GUEST is not set$/CONFIG_ASTER_GUEST=y/' .config
make ARCH=x86_64 BOARD=qemu all
make ARCH=x86_64 BOARD=qemu run-asterinas
```

## Notes

The default build pins Asterinas at `f7ff85597d892ec7476489216672b0ad61b7090f`
unless `ASTER_REF` is set. `scripts/bzimage-info.py` prints the setup header
size and derived load address for the image being staged.

Performance measurements are intended for KVM. `KVM=0 scripts/run.sh` can boot
under TCG for functional checks, but those timings are not comparable.
