#!/bin/sh
# Stage zone1 deploy artifacts on the CI host; the board pulls them via scp.
# Zone0 boot artifacts (hvisor.bin, Image, dtbs) belong in tftp_dir, not here.

set -eux

ARCH=${ARCH:?ARCH is required}
BOARD=${BOARD:?BOARD is required}
WORKSPACE_ROOT=${WORKSPACE_ROOT:-$(pwd)}
HVISOR_TOOL_PATH=${HVISOR_TOOL_PATH:-${WORKSPACE_ROOT}/hvisor-tool}
STAGING_DIR=${STAGING_DIR:-/home/light/ci_deploy}

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

if [ -z "${ZONE1_DTB:-}" ]; then
    for candidate in \
        "${IMAGE_DIR}/dts/zone1-linux.dtb" \
        "${IMAGE_DIR}/dts/linux2.dtb"; do
        if [ -f "${candidate}" ]; then
            ZONE1_DTB="${candidate}"
            break
        fi
    done
    ZONE1_DTB=${ZONE1_DTB:-${IMAGE_DIR}/dts/zone1-linux.dtb}
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

cp "${HVISOR_TOOL_PATH}/output/hvisor.ko" "${STAGING_DIR}/"
gzip -c "${HVISOR_TOOL_PATH}/output/hvisor" > "${STAGING_DIR}/hvisor.gz"
cp "${CONFIGS_DIR}/"* "${STAGING_DIR}/"
cp "${ZONE1_BOOT_SCRIPT}" "${STAGING_DIR}/"

if [ -f "${ZONE1_DTB}" ]; then
    cp "${ZONE1_DTB}" "${STAGING_DIR}/"
else
    echo "warning: zone1 dtb unavailable, skip copying ${ZONE1_DTB}"
fi

if [ -f "${CHECK_SERIAL_SCRIPT}" ]; then
    cp "${CHECK_SERIAL_SCRIPT}" "${STAGING_DIR}/"
fi

chmod -R a+rX "${STAGING_DIR}"
chmod +x "${STAGING_DIR}/boot_zone1.sh"

if [ "$(id -u)" -eq 0 ] && id light >/dev/null 2>&1; then
    chown -R light:light "${STAGING_DIR}"
fi

DEPLOY_SPLIT_CHUNK_BYTES=${DEPLOY_SPLIT_CHUNK_BYTES:-524288}
for staged_file in "${STAGING_DIR}"/*; do
    [ -f "${staged_file}" ] || continue
    staged_name=$(basename "${staged_file}")
    case "${staged_name}" in
        *.part.*) continue ;;
    esac
    size=$(stat -c%s "${staged_file}")
    if [ "${size}" -gt "${DEPLOY_SPLIT_CHUNK_BYTES}" ]; then
        split -b "${DEPLOY_SPLIT_CHUNK_BYTES}" "${staged_file}" "${STAGING_DIR}/${staged_name}.part."
        rm -f "${staged_file}"
        echo "split ${staged_name}: ${size} bytes into ${DEPLOY_SPLIT_CHUNK_BYTES}-byte chunks"
    fi
done

echo "board staging completed: ${STAGING_DIR}"
ls -la "${STAGING_DIR}"
