#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <bzImage> <output-dir>" >&2
    exit 2
fi

BZIMAGE="$1"
OUT_DIR="$2"

if [ ! -f "$BZIMAGE" ]; then
    echo "error: bzImage not found: $BZIMAGE" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"

SETUP_SECTS="$(od -An -t u1 -j 0x1f1 -N 1 "$BZIMAGE" | tr -d ' ')"
if [ -z "$SETUP_SECTS" ] || [ "$SETUP_SECTS" -eq 0 ]; then
    SETUP_SECTS=4
fi

SETUP_SIZE=$(( (SETUP_SECTS + 1) * 512 ))
SETUP_OUT="$OUT_DIR/asterinas-setup.bin"
VMLINUX_OUT="$OUT_DIR/asterinas-vmlinux.bin"

dd if="$BZIMAGE" of="$SETUP_OUT" bs=1 count="$SETUP_SIZE" status=none
dd if="$BZIMAGE" of="$VMLINUX_OUT" bs=1 skip="$SETUP_SIZE" status=none

BOOT_PROTO="$(od -An -t x2 -j 0x206 -N 2 "$SETUP_OUT" | tr -d ' ')"
CODE32_START="$(od -An -t x4 -j 0x214 -N 4 "$SETUP_OUT" | tr -d ' ')"

echo "setup_sects=$SETUP_SECTS"
echo "setup_size=$SETUP_SIZE"
echo "setup=$SETUP_OUT"
echo "vmlinux=$VMLINUX_OUT"
echo "boot_proto_version=0x$BOOT_PROTO"
echo "code32_start=0x$CODE32_START"

if [ "$((16#$BOOT_PROTO))" -lt "$((16#0204))" ]; then
    echo "WARNING: Linux boot protocol version is lower than 0x0204." >&2
fi

if [ "$CODE32_START" != "00100000" ]; then
    echo "WARNING: code32_start is not 0x00100000; update zone arch_config.kernel_entry_gpa if needed." >&2
fi
