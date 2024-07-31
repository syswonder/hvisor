# Basic settings
ARCH ?= loongarch64
LOG ?= debug
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

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

# Build paths
build_path := target/$(RUSTC_TARGET)/$(MODE)
hvisor_elf := $(build_path)/hvisor
hvisor_bin := $(build_path)/hvisor.bin
image_dir  := images/$(ARCH)

# Features based on STATS
features := 

# Build arguments
build_args := --features "$(features)" 
build_args := --target $(RUSTC_TARGET)
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
	rust-objdump --disassemble $(hvisor_elf) > hvisor.S

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
		-ex 'target remote:1234' \

clean:
	cargo clean

ifeq ($(ARCH),loongarch64)
include scripts/3a5000-loongarch64.mk
else
include scripts/qemu-$(ARCH).mk
endif