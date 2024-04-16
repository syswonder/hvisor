# Basic settings
ARCH ?= aarch64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
KDIR ?= ../../linux

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
export KDIR

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
.PHONY: all elf disa run gdb monitor clean tools rootfs transfer
all: $(hvisor_bin)

elf:
	cargo build $(build_args)

disa:
	aarch64-none-elf-readelf -a $(hvisor_elf) > hvisor-elf.txt
	rust-objdump --disassemble $(hvisor_elf) > hvisor.S

tools: 
	make -C tools && \
	make -C driver

transfer:
	~/trans_file.sh ./tools/hvisor 
	~/trans_file.sh ./driver/main.ko 

run: all tools transfer
	$(QEMU) $(QEMU_ARGS)

gdb: all tools transfer
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
	make -C tools clean
	make -C driver clean

include scripts/qemu-$(ARCH).mk
