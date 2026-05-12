#!/bin/bash
set -e
set -x

WORKSPACE_ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
PERF_DIR="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/test/perftest"
ROOTFS_EXT4="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/image/virtdisk/rootfs1.ext4"
ROOTFS_MNT="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/image/virtdisk/rootfs"
ROOTFS2_EXT4="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/image/virtdisk/rootfs2.ext4"
ROOTFS2_MNT="${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/image/virtdisk/rootfs2"
ROOTFS2_IMAGE=""

bench_tools=(fio rt-tests stress-ng iperf3 iproute2)

# Cleanup: unmount bind mounts and rootfs on any exit (error or normal)
cleanup() {
    sudo umount "${ROOTFS_MNT}/var/cache/apt/archives" 2>/dev/null || true
    sudo umount "${ROOTFS_MNT}/var/lib/apt/lists"      2>/dev/null || true
    sudo umount "${ROOTFS_MNT}/sys"  2>/dev/null || true
    sudo umount "${ROOTFS_MNT}/proc" 2>/dev/null || true
    sudo umount "${ROOTFS_MNT}/dev"  2>/dev/null || true
    [ -f "${ROOTFS_MNT}/usr/bin/qemu-riscv64-static" ] && \
        sudo rm -f "${ROOTFS_MNT}/usr/bin/qemu-riscv64-static" || true
    sudo umount "${ROOTFS2_MNT}/var/cache/apt/archives" 2>/dev/null || true
    sudo umount "${ROOTFS2_MNT}/var/lib/apt/lists"      2>/dev/null || true
    sudo umount "${ROOTFS2_MNT}/sys"  2>/dev/null || true
    sudo umount "${ROOTFS2_MNT}/proc" 2>/dev/null || true
    sudo umount "${ROOTFS2_MNT}/dev"  2>/dev/null || true
    [ -f "${ROOTFS2_MNT}/usr/bin/qemu-riscv64-static" ] && \
        sudo rm -f "${ROOTFS2_MNT}/usr/bin/qemu-riscv64-static" || true
    sudo umount "${ROOTFS2_MNT}" 2>/dev/null || true
    sudo umount "${ROOTFS_MNT}" 2>/dev/null || true
}
trap cleanup EXIT

# Remove stale mounts before invoking systemtest deploy.
cleanup

resolve_rootfs2_image() {
    if [ -f "${ROOTFS2_EXT4}" ]; then
        ROOTFS2_IMAGE="${ROOTFS2_EXT4}"
    elif [ -f "${ROOTFS_MNT}/home/riscv64/riscv_rootfs2.img" ]; then
        ROOTFS2_IMAGE="${ROOTFS_MNT}/home/riscv64/riscv_rootfs2.img"
    elif [ -f "${ROOTFS_MNT}/home/riscv64/rootfs2.ext4" ]; then
        ROOTFS2_IMAGE="${ROOTFS_MNT}/home/riscv64/rootfs2.ext4"
    elif [ -f "${ROOTFS_MNT}/rootfs2.ext4" ]; then
        ROOTFS2_IMAGE="${ROOTFS_MNT}/rootfs2.ext4"
    else
        ROOTFS2_IMAGE=""
    fi

    if [ -n "${ROOTFS2_IMAGE}" ]; then
        echo "=== rootfs2 image resolved: ${ROOTFS2_IMAGE} ==="
    fi
}

# Expand an ext2/ext3/ext4 image file in-place if free space is below min_free_mb.
# Must be called while the image is NOT mounted.
expand_ext2_if_needed() {
    local img="$1"
    local min_free_mb="${2:-1024}"

    local info block_size free_blocks free_mb
    info=$(sudo dumpe2fs -h "${img}" 2>/dev/null) || {
        echo "WARN: cannot read ${img}, skipping expand" >&2
        return 0
    }
    block_size=$(echo "${info}" | awk '/^Block size:/{print $3}')
    free_blocks=$(echo "${info}" | awk '/^Free blocks:/{gsub(/,/,"",$3); print $3}')
    if [ -z "${block_size}" ] || [ -z "${free_blocks}" ]; then
        echo "WARN: could not parse free space in ${img}, skipping expand" >&2
        return 0
    fi
    free_mb=$(( (block_size * free_blocks) / 1024 / 1024 ))
    echo "=== ${img}: ${free_mb}MB free (need ${min_free_mb}MB) ==="
    if [ "${free_mb}" -ge "${min_free_mb}" ]; then
        return 0
    fi

    local add_mb=$(( min_free_mb - free_mb + 128 ))
    echo "=== Expanding ${img} by +${add_mb}MB ==="
    sudo truncate --size=+${add_mb}M "${img}"
    sudo e2fsck -f -y "${img}" || true
    sudo resize2fs "${img}"
    echo "=== Expansion done ==="
}

ensure_tools_in_rootfs() {
    local mnt="$1"
    local mode="${2:-strict}"

    if [ -f "${mnt}/usr/bin/fio" ] && \
       [ -f "${mnt}/usr/bin/cyclictest" ] && \
       [ -f "${mnt}/usr/bin/stress-ng" ] && \
       [ -f "${mnt}/usr/bin/iperf3" ]; then
        echo "=== Benchmarking tools already installed in ${mnt}, skipping apt install ==="
        return 0
    fi

    echo "=== Installing benchmarking tools in ${mnt} via chroot (mode=${mode}) ==="
    local qemu_static="/usr/bin/qemu-riscv64-static"
    local rc=0

    # Use host tmpdirs for apt lists/cache to avoid filling guest image space.
    local apt_lists_tmp apt_cache_tmp
    apt_lists_tmp=$(mktemp -d)
    apt_cache_tmp=$(mktemp -d)

    if [ -f "${qemu_static}" ]; then
        sudo cp "${qemu_static}" "${mnt}/usr/bin/qemu-riscv64-static"
    fi
    sudo mount --bind /dev  "${mnt}/dev"
    sudo mount --bind /proc "${mnt}/proc"
    sudo mount --bind /sys  "${mnt}/sys"
    sudo mkdir -p "${mnt}/var/lib/apt/lists" "${mnt}/var/cache/apt/archives"
    sudo mount --bind "${apt_lists_tmp}" "${mnt}/var/lib/apt/lists"
    sudo mount --bind "${apt_cache_tmp}" "${mnt}/var/cache/apt/archives"

    set +e
    sudo chroot "${mnt}" sh -c \
        "apt-get update && apt-get install -y --no-install-recommends ${bench_tools[*]}"
    rc=$?
    set -e

    sudo umount "${mnt}/var/cache/apt/archives" 2>/dev/null || true
    sudo umount "${mnt}/var/lib/apt/lists" 2>/dev/null || true
    rm -rf "${apt_lists_tmp}" "${apt_cache_tmp}"
    sudo umount "${mnt}/sys" 2>/dev/null || true
    sudo umount "${mnt}/proc" 2>/dev/null || true
    sudo umount "${mnt}/dev" 2>/dev/null || true
    sudo rm -f "${mnt}/usr/bin/qemu-riscv64-static"

    if [ $rc -ne 0 ]; then
        if [ "${mode}" = "optional" ]; then
            echo "WARN: apt install failed in ${mnt}, continue with existing tools / benchmark fallback." >&2
            return 0
        fi
        echo "ERROR: apt install failed in ${mnt}" >&2
        return $rc
    fi

    echo "=== chroot install done for ${mnt} ==="
}

# Step 1: run the original systemtest deploy (hvisor, hvisor.ko, dtb, json, test scripts)
echo "=== Running base systemtest deploy ==="
"${WORKSPACE_ROOT}/platform/riscv64/qemu-plic/test/systemtest/trootfs_deploy.sh"

# Step 2: mount rootfs again and deploy bench scripts + install tools
echo "=== Mounting rootfs ==="
sudo mkdir -p "${ROOTFS_MNT}"
sudo mount "${ROOTFS_EXT4}" "${ROOTFS_MNT}"

# Step 3: install benchmarking tools for zone0/rootfs1.
ensure_tools_in_rootfs "${ROOTFS_MNT}"

# Step 4: deploy zone0 bench scripts into rootfs1
echo "=== Deploying zone0 perf bench scripts ==="
BENCH_DEST="${ROOTFS_MNT}/home/riscv64/test/bench"
sudo mkdir -p "${BENCH_DEST}"
sudo cp -v "${PERF_DIR}/bench_mem.sh" "${PERF_DIR}/bench_irq.sh" "${PERF_DIR}/bench_net.sh" "${BENCH_DEST}/"
sudo chmod +x "${BENCH_DEST}"/bench_*.sh
sudo mkdir -p "${ROOTFS_MNT}/home/riscv64/test/perfresult"

echo "=== rootfs1 perf scripts deployed ==="
sudo find "${ROOTFS_MNT}/home/riscv64/test" -ls

# Step 5: mount rootfs2 and deploy zone1 blk bench script + install tools
resolve_rootfs2_image
if [ -n "${ROOTFS2_IMAGE}" ]; then
    expand_ext2_if_needed "${ROOTFS2_IMAGE}"
    echo "=== Mounting rootfs2 (${ROOTFS2_IMAGE}) ==="
    sudo mkdir -p "${ROOTFS2_MNT}"
    sudo mount "${ROOTFS2_IMAGE}" "${ROOTFS2_MNT}"

    # Install benchmarking tools in rootfs2 (optional, may fail for small images)
    ensure_tools_in_rootfs "${ROOTFS2_MNT}" optional

    # Deploy zone1 blk bench script
    echo "=== Deploying zone1 blk perf script to rootfs2 ==="
    sudo mkdir -p "${ROOTFS2_MNT}/home/riscv64/test/bench"
    sudo mkdir -p "${ROOTFS2_MNT}/home/riscv64/test/perfresult"
    sudo cp -v "${PERF_DIR}/bench_blk.sh" "${ROOTFS2_MNT}/home/riscv64/test/bench/"
    sudo chmod +x "${ROOTFS2_MNT}/home/riscv64/test/bench/bench_blk.sh"
    sudo find "${ROOTFS2_MNT}/home/riscv64/test" -ls

    sudo umount "${ROOTFS2_MNT}"
else
    echo "WARN: rootfs2 image not found, skipping zone1 blk bench deployment"
fi

sudo umount "${ROOTFS_MNT}"

trap - EXIT
echo "=== perftest rootfs deploy done ==="
