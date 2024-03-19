# Basic settings
ARCH ?= riscv64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

export MODE
export LOG
export ARCH
export STATS

# Build paths
build_path := target/riscv64gc-unknown-none-elf/$(MODE)
target_elf := $(build_path)/hvisor
target_bin := $(build_path)/hvisor.bin

zone0_dtb    := imgs/dts/zone0.dtb
zone0_kernel := imgs/Image
zone1_dtb    := imgs/dts/zone1.dtb
zone1_kernel := imgs/Image

# Features based on STATS
features :=
ifeq ($(STATS), on)
  features += --features stats
endif

# Build arguments
build_args := --features "$(features)" -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  build_args += --release
endif

# Targets
.PHONY: all elf scp disa run gdb monitor clean update-img
all: $(target_bin)

elf:
	cargo build $(build_args)

scp: $(target_bin)
	scp -P $(PORT) -r $(target_bin) qemu-test/guest/* scp root@localhost:~/

disa:
	rust-objdump --disassemble $(target_elf) > hvisor.S

$(target_bin): elf
	$(OBJCOPY) $(target_elf) --strip-all -O binary $@

update-img:
	make -C imgs/dts

# Run targets
run: all update-img
	$(QEMU) $(QEMU_ARGS)

gdb: all update-img
	$(QEMU) $(QEMU_ARGS) -s -S

monitor:
	riscv64-unknown-elf-gdb \
		-ex 'file $(target_elf)' \
		-ex 'add-symbol-file tenants/os' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote:1234' \
		
clean:
	cargo clean

include scripts/qemu-riscv64.mk

# -drive if=none,file=fsimg1,id=hd1,format=raw

# echo " go 0x7fc00000 " | \
# -bios imgs/u-boot/u-boot.bin \
# -append "root=/dev/vda mem=768M"
# -device loader,file="$(target_bin)",addr=0x7fc00000,force-raw=on\
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

