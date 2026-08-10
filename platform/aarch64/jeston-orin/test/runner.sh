#!/bin/bash

set -e

PWD=$(pwd)
THIS=$(basename $0)
CARGO_BUILD_INPUT_ARG0=$1

ARCH=${ARCH}
BOARD=${BOARD}

UBOOT_GICV3=u-boot-atf.bin

HVISOR_ELF=$CARGO_BUILD_INPUT_ARG0
HVISOR_BIN_TMP=$HVISOR_ELF.bin.tmp
HVISOR_BIN=$HVISOR_ELF.bin

OBJCOPY=rust-objcopy

YELLOW='\033[1;33m'
END='\033[0m'

info() {
    # echo "${YELLOW}[INFO | $THIS] $1${END}"
    echo "[INFO | $THIS] $1"
}

# check if mkimage is installed
if ! command -v mkimage &>/dev/null; then
    if command -v apt &>/dev/null; then
        sudo apt update && sudo apt install -y u-boot-tools
    elif command -v brew &>/dev/null; then
        brew install u-boot-tools
    else
        info "You need to install u-boot-tools to run this script (mkimage)"
        exit 1
    fi
fi

info "Running cargo test with env: ARCH=$ARCH, BOARD=$BOARD"

info "Building hvisor with $CARGO_BUILD_INPUT_ARG0"
info "PWD=$PWD, running cargo test"
$OBJCOPY $HVISOR_ELF --strip-all -O binary $HVISOR_BIN_TMP

if [ "$ARCH" == "aarch64" ]; then
    mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
        -e 0x40400000 -d $HVISOR_BIN_TMP $HVISOR_BIN

    info "Running QEMU with $HVISOR_BIN"

    AARCH64_GIC_TEST_VERSION=3
    info "Using GIC version: $AARCH64_GIC_TEST_VERSION"

    UBOOT=$UBOOT_GICV3
    UBOOT=$PWD/platform/$ARCH/$BOARD/image/bootloader/$UBOOT
    info "Using U-Boot: $UBOOT"


    qemu-system-aarch64 \
        -machine virt,secure=on,gic-version=${AARCH64_GIC_TEST_VERSION},virtualization=on,iommu=smmuv3 \
        -cpu cortex-a57 -smp 4 -m 3G -nographic \
        -semihosting \
        -bios $UBOOT \
        -drive if=pflash,format=raw,index=1,file=flash.img \
        -device loader,file=$HVISOR_BIN,addr=0x40400000,force-raw=on \
        -global arm-smmuv3.stage=2

    exit 0
elif [ "$ARCH" == "riscv64" ]; then
    info "riscv64 auto test is not supported yet"
    exit 1
else
    info "Unsupported ARCH: $ARCH"
    exit 1
fi
