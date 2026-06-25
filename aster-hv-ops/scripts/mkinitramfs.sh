#!/bin/bash
# SPDX-License-Identifier: MulanPSL-2.0
# Build a self-contained busybox initramfs (newc cpio + gzip) for the Asterinas
# root zone. GRUB inflates the gzipped image on load, so the kernel receives a
# raw cpio whose size is reported to the guest via the board initrd window.
#
# Usage:
#   mkinitramfs.sh [output.cpio]
# Environment:
#   MODE=interactive   init mounts the pseudo filesystems then drops to a shell
#   MODE=selftest      init runs a fixed functional-verification sequence and
#                      powers off (used by the verify/bench flows)
#   BUSYBOX=/path/to/busybox-static   override the busybox binary
set -eu

OUT="$(readlink -f "${1:-initramfs.cpio}")"
MODE="${MODE:-interactive}"
# `|| true` keeps `set -e` from aborting before the diagnostic below when busybox
# is absent (BB is then empty and the check reports it).
BB="${BUSYBOX:-$(command -v busybox || true)}"

if [ -z "$BB" ] || ! file "$BB" 2>/dev/null | grep -q 'statically linked'; then
    echo "mkinitramfs: a statically linked busybox is required (set BUSYBOX=...)" >&2
    exit 1
fi

ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT
mkdir -p "$ROOT"/{bin,sbin,etc,root,proc,sys,tmp,dev,dev/pts,dev/shm,var,mnt}
install -m 0755 "$BB" "$ROOT/bin/busybox"

# Applet symlinks. Asterinas resolves these once /dev is available; the init
# script also runs `busybox --install` so later sessions find the full set.
for applet in sh ash ls cat echo mount umount mkdir rmdir cp mv rm ln ps kill \
    pwd cut grep sed awk head tail wc sort uniq dd sleep sync uname hostname \
    dmesg free clear env date true false chmod chown touch find xargs vi more \
    less od hexdump stat df du id printf seq basename dirname tr poweroff halt; do
    ln -sf busybox "$ROOT/bin/$applet"
done

printf 'root:x:0:0:root:/root:/bin/sh\n' > "$ROOT/etc/passwd"
printf 'root:x:0:\n' > "$ROOT/etc/group"

emit_prologue() {
    cat <<'EOS'
#!/bin/sh
/bin/busybox --install -s /bin 2>/dev/null
mount -t proc   proc /proc    2>/dev/null
mount -t sysfs  sys  /sys     2>/dev/null
mount -t devpts pts  /dev/pts 2>/dev/null
EOS
}

if [ "$MODE" = selftest ]; then
    { emit_prologue; cat <<'EOS'
echo "================ ASTERINAS-ON-HVISOR FUNCTIONAL VERIFICATION ================"
echo "## uname";    uname -a
echo "## cpu";      grep -cE '^processor' /proc/cpuinfo | sed 's/^/online cpus: /'
echo "## mem";      head -3 /proc/meminfo
echo "## mounts";   mount
echo "## root dir"; ls -l /
echo "## file io";  echo "asterinas-on-hvisor-ok" > /tmp/probe; cat /tmp/probe; rm -f /tmp/probe
echo "## process";  ps
echo "## status";   head -6 /proc/self/status
echo "## random";   head -c 16 /dev/urandom | od -An -tx1
echo "## env";      echo "PATH=$PATH HOME=$HOME"
echo "## compute";  i=0; s=0; while [ $i -lt 1000 ]; do s=$((s+i)); i=$((i+1)); done; echo "sum0..999=$s"
echo "================ FUNCTIONAL VERIFICATION COMPLETE: PASS ================"
sync
poweroff -f 2>/dev/null || halt -f 2>/dev/null
exec /bin/sh
EOS
    } > "$ROOT/init"
else
    { emit_prologue; cat <<'EOS'
echo "[init] Asterinas userspace on hvisor is up"
exec /bin/sh
EOS
    } > "$ROOT/init"
fi
chmod 0755 "$ROOT/init"

( cd "$ROOT" && find . | cpio -o -H newc 2>/dev/null > "$OUT" )
gzip -9 -f -k "$OUT"
raw="$(stat -c%s "$OUT")"

# The qemu board reserves a fixed initrd window (ROOT_ZONE_INITRD_SIZE = 0x40_0000,
# 4 MiB) and GRUB inflates the gzip on load, so the *raw* cpio must fit. Fail here
# rather than boot with a silently truncated initramfs.
INITRD_WINDOW="${INITRD_WINDOW:-$((0x400000))}"
if [ "$raw" -gt "$INITRD_WINDOW" ]; then
    printf 'mkinitramfs: raw cpio is %d bytes, exceeds the %d-byte initrd window; ' \
        "$raw" "$INITRD_WINDOW" >&2
    echo "shrink the initramfs or enlarge ROOT_ZONE_INITRD_SIZE (and INITRD_WINDOW)." >&2
    exit 1
fi

printf 'mkinitramfs: mode=%s raw=%d (0x%x) gz=%d -> %s.gz\n' \
    "$MODE" "$raw" "$raw" "$(stat -c%s "$OUT.gz")" "$OUT"
