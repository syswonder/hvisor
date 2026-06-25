#!/bin/bash
# SPDX-License-Identifier: MulanPSL-2.0
# Build the Asterinas guest kernel as a Linux/x86 legacy-boot bzImage with OSDK.
# The resulting image is what the hvisor "Asterinas" GRUB entry loads as the root
# zone kernel.
#
# Usage:
#   build-asterinas.sh
# Environment:
#   ASTERINAS_SRC   path to an Asterinas checkout (cloned at ASTER_REF if unset)
#   ASTER_REF       commit to build (default: pinned below)
#   OUT_DIR         where to copy the produced bzImage (default: ../build)
set -eu

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
OPS_ROOT="$(cd -- "$HERE/.." && pwd)"
ASTER_REF="${ASTER_REF:-f7ff85597d892ec7476489216672b0ad61b7090f}"
ASTERINAS_SRC="${ASTERINAS_SRC:-$OPS_ROOT/build/asterinas}"
OUT_DIR="${OUT_DIR:-$OPS_ROOT/build}"
mkdir -p "$OUT_DIR"

if [ ! -d "$ASTERINAS_SRC/.git" ]; then
    echo "[asterinas] cloning into $ASTERINAS_SRC"
    git clone https://github.com/asterinas/asterinas.git "$ASTERINAS_SRC"
fi
git -C "$ASTERINAS_SRC" checkout -q "$ASTER_REF"

# OSDK is built from the in-tree source so the generated kernel runner references
# the in-tree ostd by path; a crates.io ostd pin would double-link the global
# allocator against aster-kernel. Install it from the pinned checkout every time
# so the OSDK matches the Asterinas commit being built (a stale cargo-osdk from a
# different ref would be used otherwise). `cargo install` is a fast no-op when the
# source is unchanged.
echo "[asterinas] installing in-tree cargo-osdk from $ASTER_REF"
( cd "$ASTERINAS_SRC" && OSDK_LOCAL_DEV=1 cargo install --path osdk --locked --force )
echo "[asterinas] cargo-osdk: $(cargo osdk --version 2>&1 | head -1)"

# OSDK needs an initramfs stub present at build time; the real guest initramfs is
# built separately by mkinitramfs.sh and supplied to hvisor as a GRUB module.
STUB="$ASTERINAS_SRC/test/initramfs/build"
if [ ! -f "$STUB/initramfs.cpio.gz" ]; then
    BB="$(command -v busybox || true)"
    [ -n "$BB" ] || { echo "[asterinas] busybox not found on PATH" >&2; exit 1; }
    mkdir -p "$STUB/stub-root/bin"
    cp "$BB" "$STUB/stub-root/bin/busybox"
    printf '#!/bin/busybox sh\nexec /bin/busybox sh\n' > "$STUB/stub-root/init"
    chmod +x "$STUB/stub-root/init"
    ( cd "$STUB/stub-root" && find . | cpio -o -H newc 2>/dev/null | gzip -9 > "$STUB/initramfs.cpio.gz" )
fi

echo "[asterinas] building bzImage (--linux-x86-legacy-boot)"
rm -rf "$ASTERINAS_SRC/target/osdk"
( cd "$ASTERINAS_SRC/kernel" \
    && cargo osdk build --grub-boot-protocol linux --linux-x86-legacy-boot --profile release )

BZ="$ASTERINAS_SRC/target/osdk/iso_root/boot/aster-kernel-osdk-bin"
[ -f "$BZ" ] || { echo "[asterinas] bzImage not found at $BZ" >&2; exit 1; }
cp "$BZ" "$OUT_DIR/aster-kernel-osdk-bin"
"$HERE/bzimage-info.py" "$OUT_DIR/aster-kernel-osdk-bin"
echo "[asterinas] done -> $OUT_DIR/aster-kernel-osdk-bin"
