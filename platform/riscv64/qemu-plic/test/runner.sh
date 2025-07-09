#!/bin/bash

set -e

PWD=$(pwd)
THIS=$(basename $0)
CARGO_BUILD_INPUT_ARG0=$1

# capture env: FEATURES, ARCH

ARCH=${ARCH}
FEATURES=${FEATURES}
BOARD=${BOARD}

HVISOR_ELF=$CARGO_BUILD_INPUT_ARG0
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
$OBJCOPY $HVISOR_ELF --strip-all -O binary $HVISOR_BIN

qemu-system-riscv64 \
    -machine virt,aclint=on \
    -bios default -cpu rv64 -smp 4 -m 4G -nographic \
    -kernel $HVISOR_BIN

exit 0
