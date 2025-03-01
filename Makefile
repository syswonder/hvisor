# Basic settings
ARCH ?= aarch64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

# AVAIABLE "FEATURES" VALUES:
# - platform_qemu, platform_zcu102, platform_imx8mp
# - gicv2, gicv3 (for aarch64)
# - plic, aia (for riscv64)
FEATURES ?= platform_qemu,gicv3

# AVAIABLE "BOARD" VALUES:
# - qemu, zcu102, 3a5000
BOARD ?= qemu

ifeq ($(ARCH),aarch64)
    RUSTC_TARGET := aarch64-unknown-none
	GDB_ARCH := aarch64
else ifeq ($(ARCH),riscv64)
	RUSTC_TARGET := riscv64gc-unknown-none-elf
	GDB_ARCH := riscv:rv64
else ifeq ($(ARCH),loongarch64)
	RUSTC_TARGET := loongarch64-unknown-none
	GDB_ARCH := loongarch64
else
$(error Unsupported ARCH value: $(ARCH))
endif

export MODE
export LOG
export ARCH
export RUSTC_TARGET
export FEATURES
export BOARD

# Build paths
build_path := target/$(RUSTC_TARGET)/$(MODE)
hvisor_elf := $(build_path)/hvisor
hvisor_bin := $(build_path)/hvisor.bin
image_dir  := images/$(ARCH)

# Build arguments
build_args := 
build_args += --features "$(FEATURES)" 
build_args += --target $(RUSTC_TARGET)
build_args += -Z build-std=core,alloc
build_args += -Z build-std-features=compiler-builtins-mem

ifeq ($(MODE), release)
  build_args += --release
endif

# Targets
.PHONY: all elf disa run gdb monitor clean tools rootfs
all: $(hvisor_bin)

elf:
	cargo build $(build_args)

disa:
	readelf -a $(hvisor_elf) > hvisor-elf.txt
# rust-objdump --disassemble $(hvisor_elf) > hvisor.S
	rust-objdump --disassemble --source $(hvisor_elf) > hvisor.S

run: all
	$(QEMU) $(QEMU_ARGS)

gdb: all
	$(QEMU) $(QEMU_ARGS) -s -S

show-features:
	rustc --print=target-features --target=$(RUSTC_TARGET)

monitor:
	gdb-multiarch \
		-ex 'file $(hvisor_elf)' \
		-ex 'set arch $(GDB_ARCH)' \
		-ex 'target remote:1234'

jlink-server:
	JLinkGDBServer -select USB -if JTAG -device Cortex-A53 -port 1234

cp:
	cp $(hvisor_bin) ~/tftp

test-pre: download-test-img
	chmod +x ./tools/cargo_test.sh
	@echo "pass"

fmt-test:
	cargo fmt --all -- --check

fmt:
	cargo fmt --all

clippy:
	cargo clippy $(build_args)

flash-img:
# run this will erase all environment for uboot, be careful
# the flash.img in repo will contains the correct bootcmd
	qemu-img create -f raw flash.img 64M

download-test-img:
# first check whether the file exists
	@if [ ! -f "flash.img" ]; then echo "\nflash.img not found, downloading...\n" && \
		wget https://github.com/enkerewpo/hvisor-uboot-env-img/releases/download/v20241227/flash.img.partial && \
		./tools/extract.sh ; \
	else echo "\nflash.img found\n"; \
	fi

test: test-pre
	cp .cargo/config .cargo/config.bak
	sed "s|___HVISOR_SRC___|$(shell pwd)|g" .cargo/config.bak > .cargo/config
	cargo test $(build_args) -vv

clean:
	cargo clean

# set the BOARD variable to "3a5000"/qemu/zcu102/imx8mp to
# include the corresponding script under the ./scripts directory
include scripts/${BOARD}-${ARCH}.mk