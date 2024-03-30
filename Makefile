# Basic settings
ARCH ?= aarch64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

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
.PHONY: all elf disa run gdb monitor clean
all: $(hvisor_bin)

elf:
	cargo build $(build_args)

disa:
	aarch64-none-elf-readelf -a $(hvisor_elf) > hvisor-elf.txt
	rust-objdump --disassemble $(hvisor_elf) > hvisor.S

# Run targets
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
		-ex 'add-symbol-file ~/packages/linux/vmlinux' \

clean:
	cargo clean

include scripts/qemu-$(ARCH).mk

# -drive if=none,file=fsimg1,id=hd1,format=raw

# echo " go 0x7fc00000 " | \
# -bios imgs/u-boot/u-boot.bin \
# -append "root=/dev/vda mem=768M"
# -device loader,file="$(hvisor_bin)",addr=0x7fc00000,force-raw=on\
# -drive file=./qemu-test/host/rootfs.qcow2,discard=unmap,if=none,id=disk,format=qcow2 \
# -drive if=none,file=fsimg,id=disk,format=raw \
# -net nic \
# -net user,hostfwd=tcp::$(PORT)-:22

# dhcp
# pci enum
# virtio scan
# virtio info

# ext4ls virtio 1
# ext4load virtio 1 0x7fc00000 hvisor.bin; go 0x7fc00000

