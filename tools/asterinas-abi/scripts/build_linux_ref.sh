#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
#
# Build the Linux 5.19 reference kernel and an autorun initramfs for QEMU.
# Everything stays under the project tree. Network egress via http(s)_proxy.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
LB="$ROOT/_work/linux-build"
KVER="${KVER:-5.19}"
KOUT="$ROOT/_build/linux-$KVER-obj"

case "$ROOT" in
    /|/root|/home|/usr|/var|/tmp) echo "refusing unsafe root: $ROOT" >&2; exit 1 ;;
esac

mkdir -p "$LB"
if [ ! -d "$LB/linux-$KVER" ]; then
    echo "[1/4] fetch Linux $KVER source"
    if [ ! -f "$LB/linux-$KVER.tar.gz" ] && [ ! -f "$LB/linux-$KVER.tar.xz" ]; then
        # aliyun mirror is fast behind the proxy; kernel.org CDN is a fallback.
        curl -sS --max-time 600 -o "$LB/linux-$KVER.tar.gz" \
            "https://mirrors.aliyun.com/linux-kernel/v${KVER%%.*}.x/linux-$KVER.tar.gz" || \
        curl -sS --max-time 900 -o "$LB/linux-$KVER.tar.xz" \
            "https://cdn.kernel.org/pub/linux/kernel/v${KVER%%.*}.x/linux-$KVER.tar.xz"
    fi
    tar xf "$LB"/linux-$KVER.tar.* -C "$LB"
fi

echo "[2/4] configure (defconfig + virtio/initrd/serial/ext)"
mkdir -p "$KOUT"
( cd "$LB/linux-$KVER" && make O="$KOUT" ARCH=x86_64 defconfig >/dev/null
  ./scripts/config --file "$KOUT/.config" \
    --enable BLK_DEV_INITRD --enable DEVTMPFS --enable DEVTMPFS_MOUNT \
    --enable SERIAL_8250 --enable SERIAL_8250_CONSOLE \
    --enable VIRTIO --enable VIRTIO_PCI --enable VIRTIO_MMIO --enable VIRTIO_BLK \
    --enable VIRTIO_CONSOLE --enable VIRTIO_NET \
    --enable EXT2_FS --enable EXT4_FS --enable TMPFS \
    --enable IPV6 --enable UNIX --enable INET --enable PROC_FS --enable SYSFS \
    --disable DEBUG_INFO --disable MODULE_SIG
  make O="$KOUT" ARCH=x86_64 olddefconfig >/dev/null )

echo "[3/4] build bzImage (-j$(nproc))"
( cd "$LB/linux-$KVER" && make O="$KOUT" ARCH=x86_64 -j"$(nproc)" bzImage ) \
    > "$KOUT/build.log" 2>&1
ls -la "$KOUT/arch/x86/boot/bzImage"

echo "[4/4] autorun initramfs stamped env=L"
ABI_AUTORUN_ENV=L ABI_AUTORUN_TIMEOUT=45 \
    sh "$ROOT/initramfs/build_initramfs.sh" "$ROOT/_build/initramfs-linux.cpio.gz" >/dev/null
echo "OK: bzImage + _build/initramfs-linux.cpio.gz"
