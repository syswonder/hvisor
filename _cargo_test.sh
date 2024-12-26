# for hvisor's unit test when running cargo test
# passed to .cargo/config
# runner = "___HVISOR_SRC___/_cargo_test.sh"
# ___HVISOR_SRC___ will be replaced dynamically by the Makefile and restored after the test
# wheatfox(enkerewpo@hotmail.com) 2024.12

PWD=$(pwd)
THIS=$(basename $0)
CARGO_BUILD_INPUT_ARG0=$1

ARCH=aarch64
UBOOT=images/aarch64/bootloader/u-boot-atf.bin
# UBOOT=u-boot.bin

HVISOR_ELF=$CARGO_BUILD_INPUT_ARG0
HVISOR_BIN_TMP=$HVISOR_ELF.bin.tmp
HVISOR_BIN=$HVISOR_ELF.bin

OBJCOPY=rust-objcopy

YELLOW='\033[1;33m'
END='\033[0m'·

function info() {
    echo -e "${YELLOW}[INFO | $THIS] $1${END}"
}

info "Building hvisor with $CARGO_BUILD_INPUT_ARGS"
info "PWD=$PWD, running cargo test"
$OBJCOPY $HVISOR_ELF --strip-all -O binary $HVISOR_BIN_TMP
mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
	-e 0x40400000 -d $HVISOR_BIN_TMP $HVISOR_BIN

info "Running QEUM with $HVISOR_BIN"

qemu-system-aarch64 \
    -machine virt,secure=on,gic-version=3,virtualization=on,iommu=smmuv3 \
    -global arm-smmuv3.stage=2 \
    -cpu cortex-a57 -smp 4 -m 3G -nographic \
    -semihosting \
    -bios $UBOOT \
    -drive if=pflash,format=raw,index=1,file=flash.img \
    -device loader,file=$HVISOR_BIN,addr=0x40400000,force-raw=on
