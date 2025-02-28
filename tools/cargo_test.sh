#!/bin/bash

# for hvisor's unit test when running cargo test
# passed to .cargo/config
# runner = "___HVISOR_SRC___/_cargo_test.sh"
# ___HVISOR_SRC___ will be replaced dynamically by the Makefile and restored after the test
# wheatfox(wheatfox17@icloud.com) 2024.12

PWD=$(pwd)
THIS=$(basename $0)
CARGO_BUILD_INPUT_ARG0=$1

# capture env: FEATURES, ARCH

ARCH=${ARCH}
FEATURES=${FEATURES}
BOARD=${BOARD}

UBOOT_GICV3=images/aarch64/bootloader/u-boot-atf.bin
UBOOT_GICV2=images/aarch64/bootloader/u-boot-v2.bin
# UBOOT=u-boot.bin

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

info "Running cargo test with env: ARCH=$ARCH, FEATURES=$FEATURES, BOARD=$BOARD"

info "Building hvisor with $CARGO_BUILD_INPUT_ARG0"
info "PWD=$PWD, running cargo test"
$OBJCOPY $HVISOR_ELF --strip-all -O binary $HVISOR_BIN_TMP

if [ "$ARCH" == "aarch64" ]; then
    mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
        -e 0x40400000 -d $HVISOR_BIN_TMP $HVISOR_BIN

    info "Running QEMU with $HVISOR_BIN"

    # if we have gicv2,gicv3 in FEATURES, we get the number from it
    AARCH64_GIC_TEST_VERSION=3
    if [[ $FEATURES == *"gicv2"* ]]; then
        AARCH64_GIC_TEST_VERSION=2
    fi
    info "Using GIC version: $AARCH64_GIC_TEST_VERSION"

    UBOOT=$UBOOT_GICV3
    if [ $AARCH64_GIC_TEST_VERSION -eq 2 ]; then
        UBOOT=$UBOOT_GICV2
    fi
    info "Using U-Boot: $UBOOT"

    qemu-system-aarch64 \
        -machine virt,secure=on,gic-version=${AARCH64_GIC_TEST_VERSION},virtualization=on,iommu=smmuv3 \
        -global arm-smmuv3.stage=2 \
        -cpu cortex-a57 -smp 4 -m 3G -nographic \
        -semihosting \
        -bios $UBOOT \
        -drive if=pflash,format=raw,index=1,file=flash.img \
        -device loader,file=$HVISOR_BIN,addr=0x40400000,force-raw=on  

    mv .cargo/config.bak .cargo/config
    exit 0
elif [ "$ARCH" == "riscv64" ]; then
    info "riscv64 auto test is not supported yet"
    mv .cargo/config.bak .cargo/config
    exit 1
else
    info "Unsupported ARCH: $ARCH"
    mv .cargo/config.bak .cargo/config
    exit 1
fi