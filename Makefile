# Basic settings
ARCH ?= riscv64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
IRQ ?= plic

ifeq ($(ARCH),aarch64)
    RUSTC_TARGET := aarch64-unknown-none
	GDB_ARCH := aarch64
else
    ifeq ($(ARCH),riscv64)
        RUSTC_TARGET := riscv64gc-unknown-none-elf
		GDB_ARCH := riscv:rv64
    else
        $(error Unsupported ARCH value: $(ARCH))
    endif
endif

export MODE
export LOG
export ARCH

# Build paths
build_path := target/$(RUSTC_TARGET)/$(MODE)
hvisor_elf := $(build_path)/hvisor
hvisor_bin := $(build_path)/hvisor.bin
image_dir  := images/$(ARCH)
bios_elf := $(image_dir)/opensbi-1.2/build/platform/generic/firmware/fw_payload.elf

# Features based on STATS
features := 

# Build arguments
build_args := --features "$(IRQ)" 
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

run: build
	$(QEMU) $(QEMU_ARGS)

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@

build: $(hvisor_bin)
	make -C $(image_dir)/opensbi-1.2 PLATFORM=generic \
    	FW_PAYLOAD=y \
    	FW_PAYLOAD_PATH= ../../../$(hvisor_bin)

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

include scripts/qemu-$(ARCH).mk