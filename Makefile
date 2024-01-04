# Basic settings
ARCH ?= aarch64
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
build_path := target/$(ARCH)/$(MODE)
target_elf := $(build_path)/hvisor
target_bin := $(build_path)/hvisor.bin
guest_obj  := demo/helloworld_aarch64-qemu-virt.elf

# Features based on STATS
features :=
ifeq ($(STATS), on)
  features += --features stats
endif

# Build arguments
build_args := --features "$(features)" --target $(ARCH).json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  build_args += --release
endif

# Targets
.PHONY: all elf scp disa run gdb monitor
all: $(target_bin)

elf:
	cargo build $(build_args)

scp: $(target_bin)
	scp -P $(PORT) -r $(target_bin) qemu-test/guest/* scp root@localhost:~/

disa:
	rust-objdump --disassemble $(target_elf) > hvisor.S
	aarch64-none-elf-readelf -lS $(target_elf) > hvisor-elf.txt

$(target_bin): elf
	$(OBJCOPY) $(target_elf) --strip-all -O binary $@

# QEMU command template
define qemu_cmd
qemu-system-aarch64 \
	-drive file=./qemu-test/host/rootfs.qcow2,discard=unmap,if=none,id=disk,format=qcow2 \
	-device virtio-blk-device,drive=disk \
	-m 1G -serial mon:stdio \
	-kernel imgs/jmp/jmp.bin \
	-append "root=/dev/vda mem=768M" \
	-cpu cortex-a57 \
	-smp 16 -nographic \
	-machine virt,gic-version=3,virtualization=on \
	-device loader,file="$(target_bin)",addr=0x7fc00000,force-raw=on\
	-device virtio-serial-device -device virtconsole,chardev=con \
	-chardev vc,id=con \
	-net nic \
	-net user,hostfwd=tcp::$(PORT)-:22
endef
# -bios imgs/u-boot/u-boot.bin \
# -append "root=/dev/vda mem=768M"

# Run targets
run: all
	$(qemu_cmd)

gdb: all
	$(qemu_cmd) -s -S

monitor:
	gdb-multiarch \
		-ex 'target remote:1234' \
		-ex 'file $(target_elf)' \
		-ex 'add-symbol-file $(guest_obj)'
