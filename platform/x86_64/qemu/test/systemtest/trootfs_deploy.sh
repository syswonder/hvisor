#!/bin/bash
set -e
set -x            # Print commands for debugging

# ========================
# Environment Configuration
# ========================
WORKSPACE_ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
ROOTFS_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/image/virtdisk/rootfs"
LINUX_SRC_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/image/virtdisk/linux"
HVISOR_TOOL_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/image/virtdisk/hvisor-tool"
CONFIG_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/configs"
TEST_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/test/systemtest"
LINUX_KERNEL_IMAGE_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/image/kernel"
BOOT_BIN_DIR="${WORKSPACE_ROOT}/platform/x86_64/qemu/image/bootloader/out"

# ========================
# Function Definitions
# ========================

mount_rootfs() {
    echo "=== Mounting root filesystem ==="
    sudo mkdir -p "${ROOTFS_DIR}"
    if ! sudo mount rootfs1.img "${ROOTFS_DIR}"; then
        echo "ERROR: Failed to mount rootfs" >&2
        exit 1
    fi
}

prepare_sources() {
    echo "=== Cloning required repositories ==="
    git clone https://github.com/torvalds/linux -b v5.19 --depth=1 || return 1
    cd linux
    git checkout v5.19
    sudo cp -v "${CONFIG_DIR}/linux_config" ./.config
    make ARCH=x86_64 olddefconfig
    make modules_prepare
    cd ..
    git clone https://github.com/syswonder/hvisor-tool.git || return 1
    cd hvisor-tool
    # TODO: no checkout
    git checkout dev-x86_64
    cd ..
}

build_hvisor_tool() {
    echo "=== Building hvisor components ==="
    cd "${HVISOR_TOOL_DIR}"

    # Cross-compilation parameters
    make all \
        ARCH=x86_64 \
        LOG=LOG_INFO \
        KDIR="${LINUX_SRC_DIR}"
}

deploy_artifacts() {
    echo "=== Deploying build artifacts ==="
    local dest_dir="${ROOTFS_DIR}"
    local test_dest="${dest_dir}/test"
    # Copy main components
    sudo cp -v "${HVISOR_TOOL_DIR}/tools/hvisor" "${dest_dir}/"
    sudo cp -v "${HVISOR_TOOL_DIR}/driver/hvisor.ko" "${dest_dir}/"
    # Copy configurations
    sudo cp -v "${CONFIG_DIR}/zone1_linux.json" "${dest_dir}/zone1_linux.json"
    sudo cp -v "${CONFIG_DIR}/virtio_cfg.json" "${dest_dir}/virtio_cfg.json"
    # Copy test artifacts
    mkdir -p "${test_dest}"
    mkdir -p "${test_dest}/testcase"
    mkdir -p "${test_dest}/testresult"
    sudo cp -v ${TEST_DIR}/testcase/* "${test_dest}/testcase/"
    sudo cp -v "${TEST_DIR}/textract_dmesg.sh" "${test_dest}/"
    sudo cp -v "${TEST_DIR}/tresult.sh" "${test_dest}/"
    # Copy kernel images
    sudo cp -v "${LINUX_KERNEL_IMAGE_DIR}/setup.bin" "${dest_dir}/"
    sudo cp -v "${LINUX_KERNEL_IMAGE_DIR}/vmlinux.bin" "${dest_dir}/"
    # sudo cp -v "${BOOT_BIN_DIR}/boot.bin" "${dest_dir}/"
    # Verify deployment
    # echo "=== Deployed files list ==="
    # sudo find "${dest_dir}" -ls

}

umount_rootfs() {
    echo "=== Umounting rootfs ==="
    if mountpoint -q "${ROOTFS_DIR}"; then
        sudo umount "${ROOTFS_DIR}"
    fi
}

# ========================
# Main Execution Flow
# ========================
(
    cd "${WORKSPACE_ROOT}/platform/x86_64/qemu/image/virtdisk"
    
    # Setup environment
    prepare_sources
    mount_rootfs
    trap umount_rootfs EXIT TERM
    
    # Build process
    if ! build_hvisor_tool; then
        echo "ERROR: Build failed" >&2
        exit 1
    fi
    
    # Deployment
    deploy_artifacts

) || exit 1