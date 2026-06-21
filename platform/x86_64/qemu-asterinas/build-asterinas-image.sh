#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Build the initramfs and Asterinas images for the qemu-asterinas board.
#
# Output files are written to image/kernel/ for the board Makefile.
#
# Required environment:
#   ASTERINAS_DIR        path to an Asterinas checkout (v0.18.0) whose OSDK.toml
#                        sets [grub] boot_protocol = "linux"
#   VDSO_LIBRARY_DIR     path to the prebuilt linux_vdso libraries
# Optional environment:
#   ASTERINAS_TOOLCHAIN  rustup toolchain Asterinas builds with (default: read
#                        from $ASTERINAS_DIR/rust-toolchain.toml)
#   CARGO_OSDK           cargo-osdk binary to use. Install one pinned to the
#                        Asterinas checkout being built so a shared ~/.cargo/bin
#                        cannot interfere:
#                          OSDK_LOCAL_DEV=1 cargo install cargo-osdk \
#                            --path "$ASTERINAS_DIR/osdk" --root <dir> --force
#                        then set CARGO_OSDK=<dir>/bin/cargo-osdk
set -euo pipefail

board_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
hvisor_root="$(cd "$board_dir/../../.." && pwd)"
kernel_dir="$board_dir/image/kernel"
work_dir="$board_dir/image/.osdk-build"

: "${ASTERINAS_DIR:?set ASTERINAS_DIR to an Asterinas v0.18.0 checkout}"
: "${VDSO_LIBRARY_DIR:?set VDSO_LIBRARY_DIR to the prebuilt linux_vdso directory}"
toolchain="${ASTERINAS_TOOLCHAIN:-$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$ASTERINAS_DIR/rust-toolchain.toml")}"
if [ -z "$toolchain" ]; then
    echo "error: could not read the Asterinas toolchain from $ASTERINAS_DIR/rust-toolchain.toml; set ASTERINAS_TOOLCHAIN explicitly" >&2
    exit 1
fi

mkdir -p "$kernel_dir" "$work_dir"

echo ">> building initramfs"
"$board_dir/mkinitramfs.sh" "$kernel_dir/initramfs.cpio.gz"

# Reject initramfs archives that do not fit the fixed initrd window.
initrd_window=3145728 # 0x30_0000, must match board.rs
initrd_size="$(zcat "$kernel_dir/initramfs.cpio.gz" | wc -c)"
if [ "$initrd_size" -gt "$initrd_window" ]; then
    echo "error: decompressed initramfs is $initrd_size bytes, exceeds the" \
         "$initrd_window-byte initrd window; shrink it or enlarge ROOT_ZONE_INITRD_SIZE in board.rs" >&2
    exit 1
fi

# Keep the generated OSDK run-base crate on the Asterinas toolchain.
cat > "$work_dir/rust-toolchain.toml" <<EOF
[toolchain]
channel = "$toolchain"
components = ["rust-src", "rustc-dev", "llvm-tools-preview"]
targets = ["x86_64-unknown-none"]
EOF

echo ">> building Asterinas (linux-x86-legacy-boot)"
osdk="${CARGO_OSDK:-cargo-osdk}"
( cd "$ASTERINAS_DIR/kernel" && VDSO_LIBRARY_DIR="$VDSO_LIBRARY_DIR" "$osdk" osdk build \
    --output "$work_dir" \
    --initramfs "$kernel_dir/initramfs.cpio.gz" \
    --boot-method qemu-direct \
    --linux-x86-legacy-boot \
    --grub-boot-protocol linux \
    --strip-elf \
    --kcmd-args='console=ttyS0' \
    --kcmd-args='ostd.log_level=info' \
    --kcmd-args='init=/init' )

bzimage="$work_dir/aster-kernel-osdk-bin"
[ -f "$bzimage" ] || { echo "error: $bzimage not produced"; exit 1; }

echo ">> splitting bzImage"
# Run via cargo so the binary is located correctly even under a custom
# CARGO_TARGET_DIR (the target/ path is not always hvisor_root-relative).
imagebuilder_manifest="$hvisor_root/tools/imagebuilder/Cargo.toml"
cargo build --release --quiet --manifest-path "$imagebuilder_manifest"
cargo run --release --quiet --manifest-path "$imagebuilder_manifest" -- inspect "$bzimage"
cargo run --release --quiet --manifest-path "$imagebuilder_manifest" -- split "$bzimage" \
    --setup "$kernel_dir/asterinas-setup.bin" \
    --kernel "$kernel_dir/asterinas-vmlinux.bin"

echo ">> images ready in $kernel_dir"
ls -l "$kernel_dir"
