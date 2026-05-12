#!/bin/bash
# perftest wrapper for riscv64/qemu-plic
# Calls systemtest/tdownload_all.sh then ensures rootfs2.ext4 is extracted.
set -e

WORKSPACE_ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
UNZIP_DIR="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/image/virtdisk"
ROOTFS1="${UNZIP_DIR}/rootfs1.ext4"
ROOTFS2="${UNZIP_DIR}/rootfs2.ext4"

# Step 1: run the original download script (handles rootfs1.ext4 + Image)
"${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/test/systemtest/tdownload_all.sh" || true
# The systemtest script may exit non-zero if rootfs2 extraction fails; we handle that below.

# Step 2: if rootfs2.ext4 is still missing, extract it from rootfs1.ext4 ourselves
if [ ! -f "$ROOTFS2" ]; then
    echo "[perftest/tdownload_all] rootfs2.ext4 not found, extracting from rootfs1.ext4 ..."

    if [ ! -f "$ROOTFS1" ]; then
        echo "ERROR: $ROOTFS1 does not exist" >&2
        exit 1
    fi

    LOOP_MNT=$(mktemp -d /tmp/hvisor-rootfs1-mnt.XXXXXX)
    sudo mount -o loop "$ROOTFS1" "$LOOP_MNT"

    if [ ! -f "$LOOP_MNT/home/riscv64/riscv_rootfs2.img" ]; then
        sudo umount "$LOOP_MNT"
        rmdir "$LOOP_MNT"
        echo "ERROR: riscv_rootfs2.img not found inside rootfs1.ext4" >&2
        exit 1
    fi

    cp "$LOOP_MNT/home/riscv64/riscv_rootfs2.img" "$ROOTFS2"
    sudo umount "$LOOP_MNT"
    rmdir "$LOOP_MNT"
    echo "[perftest/tdownload_all] rootfs2.ext4 extracted successfully."
else
    echo "[perftest/tdownload_all] rootfs2.ext4 already exists, skipping extraction."
fi
