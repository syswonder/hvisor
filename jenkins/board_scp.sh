#!/bin/sh
# Stage zone1 test artifacts on the CI host for board pull via scp.
# Zone0 boot Image， Image and rootfs2.ext4 are large and
# persistent on the board; do not re-stage them every CI run.

set -eux

ARCH=${ARCH:?ARCH is required}
BOARD=${BOARD:?BOARD is required}
WORKSPACE_ROOT=${WORKSPACE_ROOT:-$(pwd)}
HVISOR_TOOL_PATH=${HVISOR_TOOL_PATH:-${WORKSPACE_ROOT}/hvisor-tool}
MODE=${MODE:-release}
STAGING_DIR=${STAGING_DIR:-/home/light/tftp/ci_deploy}

case "${HVISOR_TOOL_PATH}" in
    /*) ;;
    *) HVISOR_TOOL_PATH="${WORKSPACE_ROOT}/${HVISOR_TOOL_PATH}" ;;
esac

PLATFORM_DIR="${WORKSPACE_ROOT}/platform/${ARCH}/${BOARD}"
CONFIGS_DIR="${PLATFORM_DIR}/configs"
IMAGE_DIR="${PLATFORM_DIR}/image"
SCRIPTS_DIR="${PLATFORM_DIR}/scripts"
ZONE1_BOOT_SCRIPT="${SCRIPTS_DIR}/boot_zone1.sh"
CHECK_SERIAL_SCRIPT="${WORKSPACE_ROOT}/jenkins/check_serial.sh"
EXTERNAL_DIR=${EXTERNAL_DIR:-}
START_SCRIPT=${START_SCRIPT:-}
COPY_HVISOR_BIN=${COPY_HVISOR_BIN:-true}

case "${ARCH}" in
    x86_64) RUSTC_TARGET="x86_64-unknown-none" ;;
    aarch64) RUSTC_TARGET="aarch64-unknown-none" ;;
    riscv64) RUSTC_TARGET="riscv64gc-unknown-none-elf" ;;
    *)
        echo "error: unsupported ARCH: ${ARCH}"
        exit 1
        ;;
esac

BUILD_PATH="${WORKSPACE_ROOT}/target/${RUSTC_TARGET}/${MODE}"
HVISOR_BIN="${BUILD_PATH}/hvisor.bin"

if [ -n "${ZONE1_DTB:-}" ]; then
    :
elif [ -f "${IMAGE_DIR}/dts/rk3568_limit_zone1.dtb" ]; then
    ZONE1_DTB="${IMAGE_DIR}/dts/rk3568_limit_zone1.dtb"
elif [ -f "${IMAGE_DIR}/dts/zone1-linux.dtb" ]; then
    ZONE1_DTB="${IMAGE_DIR}/dts/zone1-linux.dtb"
else
    ZONE1_DTB="${IMAGE_DIR}/dts/zone1-linux.dtb"
fi

echo "ARCH: ${ARCH}"
echo "BOARD: ${BOARD}"
echo "HVISOR_TOOL_PATH: ${HVISOR_TOOL_PATH}"
echo "STAGING_DIR: ${STAGING_DIR}"

if [ ! -f "${HVISOR_TOOL_PATH}/output/hvisor" ]; then
    echo "error: hvisor tool binary not found: ${HVISOR_TOOL_PATH}/output/hvisor"
    exit 1
fi
if [ ! -f "${HVISOR_TOOL_PATH}/output/hvisor.ko" ]; then
    echo "error: hvisor.ko not found: ${HVISOR_TOOL_PATH}/output/hvisor.ko"
    exit 1
fi
if [ "${COPY_HVISOR_BIN}" != "false" ] && [ ! -f "${HVISOR_BIN}" ]; then
    echo "error: hvisor.bin not found: ${HVISOR_BIN}"
    exit 1
fi
if [ -n "${START_SCRIPT}" ]; then
    ZONE1_BOOT_SCRIPT="${START_SCRIPT}"
    ZONE1_SCRIPT_DEST="start.sh"
elif [ -n "${EXTERNAL_DIR}" ] && [ -f "${EXTERNAL_DIR}/start.sh" ]; then
    ZONE1_BOOT_SCRIPT="${EXTERNAL_DIR}/start.sh"
    ZONE1_SCRIPT_DEST="start.sh"
else
    ZONE1_SCRIPT_DEST="$(basename "${ZONE1_BOOT_SCRIPT}")"
fi
if [ ! -f "${ZONE1_BOOT_SCRIPT}" ]; then
    echo "error: zone1 start script not found: ${ZONE1_BOOT_SCRIPT}"
    exit 1
fi

if [ ! -f "${ZONE1_DTB}" ] && [ -d "${IMAGE_DIR}/dts" ]; then
    echo "zone1 dtb is missing, building from ${IMAGE_DIR}/dts"
    make -C "${IMAGE_DIR}/dts" all || true
fi

rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

cp "${HVISOR_TOOL_PATH}/output/hvisor" "${HVISOR_TOOL_PATH}/output/hvisor.ko" "${STAGING_DIR}/"
if [ "${COPY_HVISOR_BIN}" != "false" ]; then
    cp "${HVISOR_BIN}" "${STAGING_DIR}/"
fi
cp "${CONFIGS_DIR}/"* "${STAGING_DIR}/"
cp "${ZONE1_BOOT_SCRIPT}" "${STAGING_DIR}/${ZONE1_SCRIPT_DEST}"

if [ -f "${ZONE1_DTB}" ]; then
    cp "${ZONE1_DTB}" "${STAGING_DIR}/"
else
    echo "warning: zone1 dtb unavailable, skip copying ${ZONE1_DTB}"
fi

if [ -f "${CHECK_SERIAL_SCRIPT}" ]; then
    cp "${CHECK_SERIAL_SCRIPT}" "${STAGING_DIR}/"
fi

ZONE1_ASSETS_DIR=${ZONE1_ASSETS_DIR:-/home/light/tftp/zone1_assets}
if [ -n "${EXTERNAL_DIR}" ] && [ -f "${EXTERNAL_DIR}/Image" ]; then
    cp "${EXTERNAL_DIR}/Image" "${STAGING_DIR}/"
elif [ -f "${ZONE1_ASSETS_DIR}/Image" ]; then
    cp "${ZONE1_ASSETS_DIR}/Image" "${STAGING_DIR}/"
fi
if [ -n "${EXTERNAL_DIR}" ] && [ -f "${EXTERNAL_DIR}/rootfs2.ext4" ]; then
    cp "${EXTERNAL_DIR}/rootfs2.ext4" "${STAGING_DIR}/"
fi
# rk3568 rootfs2.ext4 is large and persistent on the board; do not re-stage/pull every run.

chmod -R a+rX "${STAGING_DIR}"
chmod +x "${STAGING_DIR}/${ZONE1_SCRIPT_DEST}"
echo "board staging completed: ${STAGING_DIR}"
ls -la "${STAGING_DIR}"
