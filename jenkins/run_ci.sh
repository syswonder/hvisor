#!/bin/sh
# Run one BID locally: build workspace hvisor-tool (clone if missing),
# prepare qemu rootfs (already under platform/) or deploy board TFTP,
# then start ci_runner.
# Usage: jenkins/run_ci.sh <arch/board>
#        jenkins/run_ci.sh --list

set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "${ROOT}"

if [ "${1:-}" = "--list" ] || [ "${1:-}" = "-l" ]; then
    python3 jenkins/ci_config.py list-bids
    exit 0
fi

BID=${1:?usage: $0 <arch/board>   (or $0 --list)}

eval "$(
    python3 - "${BID}" <<'PY'
import shlex
import sys
from pathlib import Path

sys.path.insert(0, str(Path("jenkins").resolve()))
from ci_config import get_bid_entry, load_ci, parse_bid

bid = sys.argv[1]
entry = get_bid_entry(load_ci(), bid)
arch, bid_board = parse_bid(bid)
board = (entry.get("platform_board") or "").strip() or bid_board
tests = entry.get("tests") or {}
build_args = entry.get("build_args") or {}
mode = (entry.get("mode") or "").strip()
kdir = str(build_args.get("KDIR") or "").strip()
tftp_dir = str(tests.get("tftp_dir") or "").strip()
zone0_image = str(tests.get("zone0_image") or "").strip()
dtbs = tests.get("zone0_dtbs") or []
if tests.get("zone0_dtb"):
    dtbs = [tests.get("zone0_dtb")]
dtb_list = [str(x).strip() for x in dtbs if str(x).strip()]

def emit(key, value):
    print(f"{key}={shlex.quote(str(value))}")

emit("MODE", mode)
emit("ARCH", arch)
emit("BOARD", board)
emit("KDIR", kdir)
emit("TFTP_DIR", tftp_dir)
emit("ZONE0_IMAGE", zone0_image)
emit("ZONE0_DTBS", "\n".join(dtb_list))
emit("SERIAL", str(tests.get("serial") or "").strip())
PY
)"

if [ -z "${MODE}" ]; then
    echo "error: BID ${BID} has no tests.mode in jenkins/ci.yaml" >&2
    exit 1
fi
if [ -z "${KDIR}" ]; then
    echo "error: BID ${BID} is missing build_args KDIR" >&2
    exit 1
fi

HVISOR_TOOL_PATH=${HVISOR_TOOL_PATH:-${ROOT}/hvisor-tool}
HVISOR_TOOL_URL=${HVISOR_TOOL_URL:-https://github.com/syswonder/hvisor-tool}
export TERM="${TERM:-xterm}"
export PYTHONDONTWRITEBYTECODE=1

append_path() {
    if [ -d "$1" ]; then
        PATH="$1:${PATH}"
    fi
}

append_path "${QEMU_PATH:-/home/light/DEMO/qemu-10.1.0/build}"
append_path "${CARGO_HOME:-/usr/local/cargo}/bin"
append_path "${RISCV_TOOLCHAIN_PATH:-/home/light/DEMO/toolchain/riscv64-glibc-ubuntu-24.04-gcc}/bin"
append_path "${AARCH64_TOOLCHAIN_PATH:-/home/light/DEMO/toolchain/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu}/bin"
append_path "${LOONGARCH64_TOOLCHAIN_PATH:-/home/light/DEMO/toolchain/loongarch_cross_tools}/bin"
export PATH

tool_arch() {
    case "$1" in
        aarch64|arm64) echo arm64 ;;
        riscv64|riscv) echo riscv ;;
        loongarch64|loongarch) echo loongarch ;;
        x86_64) echo x86_64 ;;
        *) echo "$1" ;;
    esac
}

ensure_hvisor_tool() {
    if [ -f "${HVISOR_TOOL_PATH}/Makefile" ]; then
        echo "hvisor-tool already present: ${HVISOR_TOOL_PATH}"
        return
    fi
    echo "Clone hvisor-tool from ${HVISOR_TOOL_URL} -> ${HVISOR_TOOL_PATH}"
    git clone --depth 1 --branch main "${HVISOR_TOOL_URL}" "${HVISOR_TOOL_PATH}"
}

build_hvisor_tool() {
    tarch=$(tool_arch "${ARCH}")
    echo "Build hvisor-tool [BID=${BID}, ARCH=${tarch}, KDIR=${KDIR}]"
    make -C "${HVISOR_TOOL_PATH}" all ARCH="${tarch}" KDIR="${KDIR}"
    test -f "${HVISOR_TOOL_PATH}/output/hvisor"
    test -f "${HVISOR_TOOL_PATH}/output/hvisor.ko"
}

prepare_qemu() {
    platform_dir="${ROOT}/platform/${ARCH}/${BOARD}"
    virtdisk="${platform_dir}/image/virtdisk"
    if [ ! -f "${virtdisk}/rootfs1.ext4" ] && [ ! -f "${virtdisk}/rootfs1.img" ]; then
        echo "error: qemu rootfs missing under ${virtdisk} (expected rootfs1.ext4 or rootfs1.img)" >&2
        exit 1
    fi
    echo "Prepare qemu rootfs [BID=${BID}, ARCH=${ARCH}, BOARD=${BOARD}]"
    sudo -E env \
        ARCH="${ARCH}" \
        BOARD="${BOARD}" \
        KDIR="${KDIR}" \
        WORKSPACE_ROOT="${ROOT}" \
        HVISOR_TOOL_PATH="${HVISOR_TOOL_PATH}" \
        "${ROOT}/jenkins/prepare.sh"
}

deploy_board_tftp() {
    if [ -z "${TFTP_DIR}" ]; then
        echo "error: BID ${BID} is missing tests.tftp_dir" >&2
        exit 1
    fi
    zone0_image="${ZONE0_IMAGE}"
    if [ -z "${zone0_image}" ]; then
        zone0_image="${KDIR}/arch/arm64/boot/Image"
    fi
    echo "Deploy TFTP [BID=${BID}, TFTP_DIR=${TFTP_DIR}]"
    tftp_staging="${ROOT}/.tftp-staging"
    rm -rf "${tftp_staging}"
    make cp ARCH="${ARCH}" BOARD="${BOARD}" MODE=release TFTP_DIR="${tftp_staging}"
    test -f "${tftp_staging}/hvisor.bin"
    sudo mkdir -p "${TFTP_DIR}"
    sudo find "${TFTP_DIR}" -mindepth 1 -maxdepth 1 -type f -delete
    sudo cp "${tftp_staging}/hvisor.bin" "${TFTP_DIR}/"
    test -f "${TFTP_DIR}/hvisor.bin"

    if [ -n "${ZONE0_DTBS}" ]; then
        printf '%s\n' "${ZONE0_DTBS}" | while IFS= read -r dtb; do
            [ -n "${dtb}" ] || continue
            test -f "${dtb}"
            sudo cp "${dtb}" "${TFTP_DIR}/"
        done
    fi

    test -f "${zone0_image}" || {
        echo "error: zone0 kernel Image not found: ${zone0_image}" >&2
        exit 1
    }
    sudo cp "${zone0_image}" "${TFTP_DIR}/Image"
    sudo chmod -R a+rX "${TFTP_DIR}"
    ls -la "${TFTP_DIR}"
}

free_board_serial() {
    echo "Free board serial [BID=${BID}]"
    # Kill leftover interactive consoles / prior ci_runner only (not this run_ci.sh).
    sudo pkill -f "jenkins/ci_runner.py --bid ${BID}" 2>/dev/null || true
    sudo pkill -f "screen ${SERIAL}" 2>/dev/null || true
    if [ -n "${SERIAL}" ] && [ -e "${SERIAL}" ]; then
        resolved=$(readlink -f "${SERIAL}" 2>/dev/null || true)
        if [ -n "${resolved}" ]; then
            sudo pkill -f "screen ${resolved}" 2>/dev/null || true
            sudo pkill -f "picocom.*${resolved}" 2>/dev/null || true
        fi
        echo "  fuser -k ${SERIAL}"
        sudo fuser -k "${SERIAL}" 2>/dev/null || true
        if [ -n "${resolved}" ] && [ "${resolved}" != "${SERIAL}" ] && [ -e "${resolved}" ]; then
            echo "  fuser -k ${resolved}"
            sudo fuser -k "${resolved}" 2>/dev/null || true
        fi
    else
        echo "  skip: serial empty or missing (${SERIAL:-unset})"
    fi
    sleep 1
    if [ -n "${SERIAL}" ] && [ -e "${SERIAL}" ]; then
        if sudo fuser "${SERIAL}" >/dev/null 2>&1; then
            echo "warning: ${SERIAL} still busy after free attempt" >&2
        else
            echo "  serial free: ${SERIAL}"
        fi
    fi
}

run_ci() {
    echo "Run ci_runner [BID=${BID}, mode=${MODE}]"
    if [ "${MODE}" = "board" ]; then
        sudo -E env \
            TERM="${TERM}" \
            HVISOR_TOOL_PATH="${HVISOR_TOOL_PATH}" \
            python3 "${ROOT}/jenkins/ci_runner.py" --bid "${BID}"
    else
        python3 "${ROOT}/jenkins/ci_runner.py" --bid "${BID}"
    fi
}

check_log_severity() {
    echo "Check console log severity [BID=${BID}]"
    python3 "${ROOT}/jenkins/check_log_severity.py" --bid "${BID}"
}

case "${MODE}" in
    qemu)
        ensure_hvisor_tool
        build_hvisor_tool
        prepare_qemu
        run_ci
        check_log_severity
        ;;
    board)
        free_board_serial
        ensure_hvisor_tool
        build_hvisor_tool
        deploy_board_tftp
        free_board_serial
        run_ci
        check_log_severity
        ;;
    *)
        echo "error: unsupported tests.mode='${MODE}' for BID ${BID}" >&2
        exit 1
        ;;
esac
