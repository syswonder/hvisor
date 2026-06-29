#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BUILD="$ROOT/_build"
STAGE="$BUILD/initramfs-root"
OUT="${1:-$BUILD/initramfs.cpio.gz}"

# Optional autorun stamping (deterministic, cmdline-independent):
#   ABI_AUTORUN_ENV=<A|B|C>  -> stamp /etc/abi-autorun + /etc/abi-env
#   ABI_AUTORUN_TIMEOUT=<s>   -> stamp /etc/abi-timeout (default 45)
ABI_AUTORUN_ENV="${ABI_AUTORUN_ENV:-}"
ABI_AUTORUN_TIMEOUT="${ABI_AUTORUN_TIMEOUT:-45}"

case "$ROOT" in
    /|/root|/home|/usr|/var|/tmp)
        echo "refusing unsafe project root: $ROOT" >&2
        exit 1
        ;;
esac

rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/dev" "$STAGE/proc" "$STAGE/sys" "$STAGE/tmp" "$STAGE/run" "$STAGE/usr/bin" "$STAGE/etc"
chmod 1777 "$STAGE/tmp"

if [ -n "$ABI_AUTORUN_ENV" ]; then
    : > "$STAGE/etc/abi-autorun"
    # Terminate with a newline: busybox `read` returns non-zero at EOF without
    # one, which would make the init fall back to an "unknown" environment id.
    printf '%s\n' "$ABI_AUTORUN_ENV" > "$STAGE/etc/abi-env"
    printf '%s\n' "$ABI_AUTORUN_TIMEOUT" > "$STAGE/etc/abi-timeout"
    echo "initramfs: autorun stamped for env=$ABI_AUTORUN_ENV timeout=${ABI_AUTORUN_TIMEOUT}s" >&2
fi

if command -v busybox >/dev/null 2>&1; then
    busybox_path="$(command -v busybox)"
    if ldd "$busybox_path" >/dev/null 2>&1; then
        echo "WARNING: busybox appears dynamically linked; guest shell may need shared libraries" >&2
        ldd "$busybox_path" > "$BUILD/busybox.ldd.txt" 2>&1 || true
    fi
    cp "$busybox_path" "$STAGE/bin/busybox"
    (cd "$STAGE/bin" && for applet in sh mount umount mkdir rmdir cat echo ls grep egrep \
        sed sort find xargs cp mv rm chmod chown date id uname sleep od seq wc tr env kill \
        sync poweroff halt reboot ps dmesg head tail wc stat ln touch true false test printf \
        readlink dd cut awk ip ifconfig route; do \
        ln -sf busybox "$applet"; done)
else
    echo "busybox not found; initramfs shell will be incomplete" >&2
fi

cp "$ROOT/initramfs/init" "$STAGE/init"
chmod +x "$STAGE/init"
cp "$ROOT/tests/run_all.sh" "$STAGE/usr/bin/run_all.sh"
chmod +x "$STAGE/usr/bin/run_all.sh"

# Compiled runner (preferred inside the guest; robust per-test timeouts).
if [ -x "$BUILD/abi_runner.test" ]; then
    cp "$BUILD/abi_runner.test" "$STAGE/usr/bin/abi_runner"
    chmod +x "$STAGE/usr/bin/abi_runner"
    [ -n "${STRIP:-}" ] && command -v "${STRIP:-strip}" >/dev/null 2>&1 && \
        "${STRIP:-strip}" --strip-unneeded "$STAGE/usr/bin/abi_runner" 2>/dev/null || true
else
    echo "WARNING: $BUILD/abi_runner.test not found; guest will fall back to run_all.sh" >&2
fi

mkdir -p "$STAGE/usr/bin/abi-tests"
STRIP="${STRIP:-strip}"
command -v "$STRIP" >/dev/null 2>&1 || STRIP=""
if [ -d "$BUILD/tests" ]; then
    (cd "$BUILD/tests" && find . -type f -name '*.test' -print | while read -r f; do
        mkdir -p "$STAGE/usr/bin/abi-tests/$(dirname "$f")"
        cp "$BUILD/tests/$f" "$STAGE/usr/bin/abi-tests/$f"
        chmod +x "$STAGE/usr/bin/abi-tests/$f"
        # Strip to shrink the initramfs: ~900 KiB -> ~50 KiB per static binary,
        # which roughly halves unpack time inside the guest.
        [ -n "$STRIP" ] && "$STRIP" --strip-unneeded "$STAGE/usr/bin/abi-tests/$f" 2>/dev/null || true
    done)
else
    echo "No compiled tests found under $BUILD/tests; run make tests first" >&2
fi

for script in "$ROOT"/tests/l5_observability/*.sh "$ROOT"/tests/real_apps/*.sh; do
    [ -f "$script" ] || continue
    rel="${script#$ROOT/tests/}"
    mkdir -p "$STAGE/usr/bin/abi-tests/$(dirname "$rel")"
    cp "$script" "$STAGE/usr/bin/abi-tests/$rel"
    chmod +x "$STAGE/usr/bin/abi-tests/$rel"
done

mkdir -p "$(dirname "$OUT")"
(cd "$STAGE" && find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$OUT")
echo "$OUT"
