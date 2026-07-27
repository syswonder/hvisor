#!/bin/sh

set -eu

ARCH=${ARCH:?ARCH is required}
BOARD=${BOARD:?BOARD is required}
WORKSPACE_ROOT=${WORKSPACE_ROOT:-$(pwd)}
HVISOR_TOOL_PATH=${HVISOR_TOOL_PATH:-${WORKSPACE_ROOT}/hvisor-tool}
TFTP_DIR=${TFTP_DIR:?TFTP_DIR is required}
SCP_SRC=${SCP_SRC:?SCP_SRC is required}
EXTERNAL_DIR=${EXTERNAL_DIR:?EXTERNAL_DIR is required}

case "${HVISOR_TOOL_PATH}" in
    /*) ;;
    *) HVISOR_TOOL_PATH="${WORKSPACE_ROOT}/${HVISOR_TOOL_PATH}" ;;
esac

PLATFORM_DIR="${WORKSPACE_ROOT}/platform/${ARCH}/${BOARD}"
DTS_DIR="${PLATFORM_DIR}/image/dts"
CONFIGS_DIR="${PLATFORM_DIR}/configs"
HVISOR_BIN="${WORKSPACE_ROOT}/target/aarch64-unknown-none/release/hvisor.bin"

require_file() {
    if [ ! -f "$1" ]; then
        echo "error: required file not found: $1" >&2
        exit 1
    fi
}

if [ ! -f "${DTS_DIR}/linux1.dtb" ] || [ ! -f "${DTS_DIR}/linux2.dtb" ] || [ ! -f "${DTS_DIR}/phytium-pi-board-v2.dtb" ]; then
    make -C "${DTS_DIR}" all
fi

require_file "${HVISOR_BIN}"
require_file "${DTS_DIR}/linux1.dtb"
require_file "${DTS_DIR}/linux2.dtb"
require_file "${DTS_DIR}/phytium-pi-board-v2.dtb"
require_file "${HVISOR_TOOL_PATH}/output/hvisor"
require_file "${HVISOR_TOOL_PATH}/output/hvisor.ko"
require_file "${CONFIGS_DIR}/zone1-linux-virtio.json"
require_file "${CONFIGS_DIR}/zone1-linux.json"
require_file "${EXTERNAL_DIR}/Image"
require_file "${EXTERNAL_DIR}/rootfs2.ext4"
require_file "${EXTERNAL_DIR}/start.sh"

mkdir -p "${TFTP_DIR}" "${SCP_SRC}"

find "${TFTP_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
find "${SCP_SRC}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +

cp "${HVISOR_BIN}" "${TFTP_DIR}/hvisor.bin"
cp "${DTS_DIR}/linux1.dtb" "${TFTP_DIR}/linux1.dtb"
cp "${DTS_DIR}/phytium-pi-board-v2.dtb" "${TFTP_DIR}/phytium-pi-board-v2.dtb"
cp "${EXTERNAL_DIR}/Image" "${TFTP_DIR}/Image"

cp "${HVISOR_TOOL_PATH}/output/hvisor" "${SCP_SRC}/hvisor"
cp "${HVISOR_TOOL_PATH}/output/hvisor.ko" "${SCP_SRC}/hvisor.ko"
cp "${DTS_DIR}/linux2.dtb" "${SCP_SRC}/linux2.dtb"
cp "${EXTERNAL_DIR}/Image" "${SCP_SRC}/Image"
cp "${EXTERNAL_DIR}/rootfs2.ext4" "${SCP_SRC}/rootfs2.ext4"
cp "${EXTERNAL_DIR}/start.sh" "${SCP_SRC}/start.sh"
cp "${CONFIGS_DIR}/zone1-linux-virtio.json" "${SCP_SRC}/zone1-linux-virtio.json"
cp "${CONFIGS_DIR}/zone1-linux.json" "${SCP_SRC}/zone1-linux.json"
chmod +x "${SCP_SRC}/start.sh"

echo "Prepared Phytium-Pi TFTP artifacts in ${TFTP_DIR}"
echo "Prepared Phytium-Pi SCP artifacts in ${SCP_SRC}"
