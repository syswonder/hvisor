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
#   LINUX_VDSO_SRC  path to a linux_vdso checkout (cloned at LINUX_VDSO_REF if unset)
#   LINUX_VDSO_REF  linux_vdso commit to build with (default: pinned below)
#   VDSO_LIBRARY_DIR
#                   path to pre-fetched vDSO binaries; overrides LINUX_VDSO_SRC
#   OUT_DIR         where to copy the produced bzImage (default: ../build)
set -eu

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
OPS_ROOT="$(cd -- "$HERE/.." && pwd)"
ASTER_REF="${ASTER_REF:-f7ff85597d892ec7476489216672b0ad61b7090f}"
LINUX_VDSO_REF="${LINUX_VDSO_REF:-74898350d406d6cd8988531ad737380a8e2cdbf4}"
ASTERINAS_SRC="${ASTERINAS_SRC:-$OPS_ROOT/build/asterinas}"
LINUX_VDSO_SRC="${LINUX_VDSO_SRC:-$OPS_ROOT/build/linux_vdso}"
OUT_DIR="${OUT_DIR:-$OPS_ROOT/build}"
OSDK_ROOT="${OSDK_ROOT:-$OPS_ROOT/build/osdk}"
mkdir -p "$OUT_DIR"

if [ ! -d "$ASTERINAS_SRC/.git" ]; then
    echo "[asterinas] cloning into $ASTERINAS_SRC"
    git clone https://github.com/asterinas/asterinas.git "$ASTERINAS_SRC"
fi
git -C "$ASTERINAS_SRC" fetch -q origin "$ASTER_REF" || git -C "$ASTERINAS_SRC" fetch -q origin
git -C "$ASTERINAS_SRC" checkout -q "$ASTER_REF"

if [ -z "${VDSO_LIBRARY_DIR:-}" ]; then
    if [ ! -d "$LINUX_VDSO_SRC/.git" ]; then
        echo "[asterinas] cloning linux_vdso into $LINUX_VDSO_SRC"
        git clone https://github.com/asterinas/linux_vdso.git "$LINUX_VDSO_SRC"
    fi
    git -C "$LINUX_VDSO_SRC" fetch -q origin "$LINUX_VDSO_REF" || git -C "$LINUX_VDSO_SRC" fetch -q origin
    git -C "$LINUX_VDSO_SRC" checkout -q "$LINUX_VDSO_REF"
    export VDSO_LIBRARY_DIR="$LINUX_VDSO_SRC"
else
    [ -f "$VDSO_LIBRARY_DIR/vdso_x86_64.so" ] || {
        echo "[asterinas] VDSO_LIBRARY_DIR does not contain vdso_x86_64.so: $VDSO_LIBRARY_DIR" >&2
        exit 1
    }
fi
echo "[asterinas] linux_vdso: $(git -C "$VDSO_LIBRARY_DIR" rev-parse --short=12 HEAD 2>/dev/null || echo external)"

# OSDK is built from the in-tree source so the generated kernel runner references
# the in-tree ostd by path; a crates.io ostd pin would double-link the allocator
# against aster-kernel. Install it under the local build directory so this helper
# does not replace a developer's global cargo-osdk.
echo "[asterinas] installing local cargo-osdk from $ASTER_REF"
( cd "$ASTERINAS_SRC" && OSDK_LOCAL_DEV=1 cargo install --path osdk --locked --force --root "$OSDK_ROOT" )
export PATH="$OSDK_ROOT/bin:$PATH"
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
{
    echo "ASTER_REF=$ASTER_REF"
    echo "LINUX_VDSO_REF=$(git -C "$VDSO_LIBRARY_DIR" rev-parse HEAD 2>/dev/null || echo external)"
    if [ "$(cd "$VDSO_LIBRARY_DIR" 2>/dev/null && pwd)" = "$(cd "$LINUX_VDSO_SRC" 2>/dev/null && pwd)" ]; then
        echo "VDSO_LIBRARY_DIR=build/linux_vdso"
    else
        echo "VDSO_LIBRARY_DIR=external"
    fi
} > "$OUT_DIR/aster-kernel-osdk-bin.meta"
"$HERE/bzimage-info.py" "$OUT_DIR/aster-kernel-osdk-bin"
echo "[asterinas] done -> $OUT_DIR/aster-kernel-osdk-bin"
