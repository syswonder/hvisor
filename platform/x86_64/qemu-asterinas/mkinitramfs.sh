#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Build a minimal busybox initramfs for the Asterinas root zone.
#
# The archive contains a statically linked busybox, the applet symlinks, and the
# init script from initramfs/init. grub loads the gzipped archive as a module and
# hvisor publishes it to the guest through the boot_params ramdisk fields.
#
# Usage: mkinitramfs.sh [output.cpio.gz]
#   BUSYBOX=/path/to/busybox   override the busybox binary (default: $(command -v busybox))
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:-$here/image/kernel/initramfs.cpio.gz}"
busybox="${BUSYBOX:-$(command -v busybox || true)}"

if [ -z "$busybox" ] || [ ! -x "$busybox" ]; then
    echo "error: a static busybox binary is required (set BUSYBOX or install busybox-static)" >&2
    exit 1
fi

# A dynamically linked busybox would need its shared libraries inside the
# initramfs; this builder only ships the binary, so require a static one.
if ldd_out="$(ldd "$busybox" 2>/dev/null)" \
    && ! printf '%s\n' "$ldd_out" | grep -qE 'not a dynamic executable|statically linked'; then
    echo "error: '$busybox' is dynamically linked; a static busybox is required (set BUSYBOX or install busybox-static)" >&2
    exit 1
fi

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

mkdir -p "$root"/{dev,etc,proc,sys,tmp,root,run,usr/bin,usr/sbin,usr/lib,usr/lib64}
ln -sfn usr/bin "$root/bin"
ln -sfn usr/sbin "$root/sbin"
ln -sfn usr/lib "$root/lib"
ln -sfn usr/lib64 "$root/lib64"

install -m 0755 "$busybox" "$root/usr/bin/busybox"
for applet in $("$busybox" --list); do
    [ "$applet" = busybox ] && continue
    ln -sf busybox "$root/usr/bin/$applet"
done

install -m 0755 "$here/initramfs/init" "$root/init"

mkdir -p "$(dirname "$out")"
# Keep the archive byte-stable for identical inputs.
find "$root" -exec touch -h -d @0 {} +
( cd "$root" && find . -mindepth 1 -print0 | LC_ALL=C sort -z \
    | cpio --null --create --format=newc --reproducible --owner=0:0 --quiet ) | gzip -9 -n > "$out"

echo "initramfs: $out ($(stat -c%s "$out") bytes)"
