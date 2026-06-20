#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
#
# Build Asterinas as an hvisor-bootable x86_64 image and stage it for GRUB.
#
# The kernel is built with OSDK under the Linux/x86 32-bit boot protocol
# (linux-legacy32), which matches hvisor's zone loader: hvisor parses the setup
# header, fills a boot_params/zeropage, and enters the protected-mode kernel at
# code32_start (0x100000). The resulting bzImage is split into a setup blob and
# a kernel payload that map onto the zone JSON's setup_filepath/kernel_filepath
# fields (here, the root zone's GRUB modules).
#
# Required environment:
#   ASTERINAS_DIR       Asterinas source checkout (tested at f0958799...)
#   VDSO_LIBRARY_DIR    asterinas/linux_vdso checkout (provides the guest vDSO)
# Optional:
#   ASTER_TOOLCHAIN     rustup toolchain (default: nightly-2026-04-03)
#   GUEST_CC            C compiler for the probe initramfs (default: musl-gcc)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HV_ROOT="$(cd "$HERE/../../../.." && pwd)"
KERNEL_DIR="$HV_ROOT/platform/x86_64/qemu/image/kernel"
ARTIFACTS="$HERE/artifacts"
TOOLCHAIN="${ASTER_TOOLCHAIN:-nightly-2026-04-03}"

: "${ASTERINAS_DIR:?set ASTERINAS_DIR to the Asterinas source checkout}"
: "${VDSO_LIBRARY_DIR:?set VDSO_LIBRARY_DIR to the asterinas/linux_vdso checkout}"
export VDSO_LIBRARY_DIR
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(nproc)}"

mkdir -p "$ARTIFACTS" "$KERNEL_DIR"

echo "[1/4] Building the probe initramfs (static musl, uncompressed CPIO)"
# Asterinas streams its bundled gzip decoder over the initrd; shipping an
# uncompressed CPIO archive is the robust contract, and musl keeps the static
# probe binaries small.
GUEST_CC="${GUEST_CC:-musl-gcc}"
CC="$GUEST_CC" "$HERE/scripts/make_guest_initramfs.sh" "$ARTIFACTS/initramfs.cpio.gz" >/dev/null
gzip -dc "$ARTIFACTS/initramfs.cpio.gz" > "$KERNEL_DIR/initramfs.cpio.gz"

echo "[2/4] Installing cargo-osdk (OSDK_LOCAL_DEV=1)"
# OSDK_LOCAL_DEV makes the generated runner crate depend on the in-tree ostd by
# path rather than crates.io, avoiding a duplicate #[global_allocator].
OSDK_LOCAL_DEV=1 cargo "+$TOOLCHAIN" install cargo-osdk \
    --path "$ASTERINAS_DIR/osdk" --locked --force

echo "[3/4] Building the Asterinas linux-legacy32 bzImage"
cat > "$ARTIFACTS/rust-toolchain.toml" <<EOF
[toolchain]
channel = "$TOOLCHAIN"
components = ["rust-src", "rustc-dev", "llvm-tools-preview"]
targets = ["x86_64-unknown-none"]
EOF
( cd "$ASTERINAS_DIR" && cargo "+$TOOLCHAIN" osdk build \
    --profile release \
    --target-arch x86_64 \
    --boot-method grub-rescue-iso \
    --grub-boot-protocol linux --linux-x86-legacy-boot \
    --initramfs "$KERNEL_DIR/initramfs.cpio.gz" \
    --output "$ARTIFACTS" )

echo "[4/4] Splitting the bzImage into setup + kernel"
BZIMAGE="$ARTIFACTS/iso_root/boot/aster-kernel-osdk-bin"
[ -s "$BZIMAGE" ] || BZIMAGE="$(find "$ARTIFACTS/iso_root/boot" -maxdepth 1 -type f \
    ! -name 'initramfs*' ! -name '*.cfg' | head -1)"
python3 "$HERE/scripts/split_bzimage.py" "$BZIMAGE" \
    --setup-out "$KERNEL_DIR/asterinas-setup.bin" \
    --kernel-out "$KERNEL_DIR/asterinas-vmlinux.bin" \
    --metadata-out "$ARTIFACTS/asterinas-bzimage-meta.json"

echo
echo "Staged for GRUB in $KERNEL_DIR:"
ls -l "$KERNEL_DIR"/asterinas-setup.bin "$KERNEL_DIR"/asterinas-vmlinux.bin \
      "$KERNEL_DIR"/initramfs.cpio.gz
