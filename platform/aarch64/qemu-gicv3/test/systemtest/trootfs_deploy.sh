#!/bin/bash
set -e
set -x            # Print commands for debugging

# ========================
# Environment Configuration
# ========================
WORKSPACE_ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
ROOTFS_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/image/virtdisk/rootfs"
LINUX_KERNEL_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/image/virtdisk/linux_5.4"
HVISOR_TOOL_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/image/virtdisk/hvisor-tool"
CONFIG_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/configs"
TEST_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/test/systemtest"
DTS_DIR="${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/image/dts"

# ========================
# Function Definitions
# ========================

mount_rootfs() {
    echo "=== Mounting root filesystem ==="
    sudo mkdir -p "${ROOTFS_DIR}"
    if ! sudo mount rootfs1.ext4 "${ROOTFS_DIR}"; then
        echo "ERROR: Failed to mount rootfs" >&2
        exit 1
    fi
}

prepare_sources() {
    echo "=== Cloning required repositories ==="
    git clone https://github.com/CHonghaohao/linux_5.4.git || return 1
    git clone https://github.com/syswonder/hvisor-tool.git || return 1
}

build_hvisor_tool() {
    echo "=== Building hvisor components ==="
    cd "${HVISOR_TOOL_DIR}"

    # Cross-compilation parameters
    make all \
        ARCH=arm64 \
        LOG=LOG_INFO \
        KDIR="${LINUX_KERNEL_DIR}"
}

deploy_artifacts() {
    echo "=== Deploying build artifacts ==="
    local dest_dir="${ROOTFS_DIR}/home/arm64"
    local test_dest="${dest_dir}/test"
    # Copy main components
    sudo cp -v "${HVISOR_TOOL_DIR}/tools/hvisor" "${dest_dir}/"
    sudo cp -v "${HVISOR_TOOL_DIR}/driver/hvisor.ko" "${dest_dir}/"
    # Device Tree & Configurations
    sudo cp -v "${DTS_DIR}/zone1-linux.dtb" "${dest_dir}/zone1-linux.dtb"
    sudo cp -v "${CONFIG_DIR}/zone1-linux.json" "${dest_dir}/zone1-linux.json"
    sudo cp -v "${CONFIG_DIR}/zone1-linux-virtio.json" "${dest_dir}/zone1-linux-virtio.json"
    # Test artifacts
    sudo cp -v ${TEST_DIR}/testcase/* "${test_dest}/testcase/"
    sudo cp -v "${TEST_DIR}/textract_dmesg.sh" "${test_dest}/"
    sudo cp -v "${TEST_DIR}/tresult.sh" "${test_dest}/"
    # Boot zone1 shells
    sudo cp -v "${TEST_DIR}/boot_zone1.sh" "${dest_dir}/"
    sudo cp -v "${TEST_DIR}/screen_zone1.sh" "${dest_dir}/"

    # Verify deployment
    echo "=== Deployed files list ==="
    sudo find "${dest_dir}" -ls

}

# ========================
# Main Execution Flow
# ========================
(
    cd "${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/image/virtdisk"
    
    # Setup environment
    mount_rootfs
    prepare_sources
    
    # Build process
    if ! build_hvisor_tool; then
        echo "ERROR: Build failed" >&2
        exit 1
    fi
    
    # Deployment
    deploy_artifacts

    # Cleanup
    echo "=== Unmounting rootfs ==="
    sudo umount "${ROOTFS_DIR}"
) || exit 1