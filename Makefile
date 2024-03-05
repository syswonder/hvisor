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

root_dtb    := imgs/dts/root.dtb
root_kernel := imgs/Image
nr1_dtb    := imgs/dts/nr1.dtb
nr1_kernel := imgs/Image

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
.PHONY: all elf scp disa run gdb monitor clean
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

FSIMG1=/path/to/disk1.img
FSIMG2=/path/to/disk2.img

# QEMU command template
define qemu_cmd
e2fsck -f $(FSIMG1) && \
e2fsck -f $(FSIMG2) && \
sudo qemu-system-aarch64 \
	-machine virt,secure=on,gic-version=3,virtualization=on \
	-cpu cortex-a57 \
	-smp 4 \
	-m 2G \
	-nographic \
	-bios imgs/u-boot.bin \
	\
	-device loader,file="$(target_bin)",addr=0x7fc00000,force-raw=on \
	-device loader,file="$(root_dtb)",addr=0x40100000,force-raw=on \
	-device loader,file="$(root_kernel)",addr=0x40200000,force-raw=on \
	-device loader,file="$(nr1_dtb)",addr=0x60100000,force-raw=on \
	-device loader,file="$(nr1_kernel)",addr=0x60200000,force-raw=on \
	\
	-drive if=none,file=$(FSIMG1),id=Xa003e000,format=raw \
	-device virtio-blk-device,drive=Xa003e000 \
	\
	-drive if=none,file=$(FSIMG2),id=Xa003c000,format=raw \
	-device virtio-blk-device,drive=Xa003c000 \
	\
	-netdev tap,id=Xa003a000,ifname=tap0,script=no,downscript=no \
	-device virtio-net-device,netdev=Xa003a000,mac=52:55:00:d1:55:01 \
	\
	-chardev pty,id=Xa0038000 \
	-device virtio-serial-device -device virtconsole,chardev=Xa0038000
endef

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

update-img:
	make -C imgs/dts

# Run targets
run: all update-img
	$(qemu_cmd)

gdb: all update-img
	$(qemu_cmd) -s -S

monitor:
	gdb-multiarch \
		-ex 'target remote:1234' \
		-ex 'file $(target_elf)' \
		-ex 'add-symbol-file $(guest_obj)' \
		-ex 'c'

clean:
	cargo clean

baremetal:
	sudo qemu-system-aarch64 \
		-machine virt,secure=on,gic-version=3,virtualization=on \
		-cpu cortex-a57 \
		-smp 4 \
		-m 2G \
		-nographic \
		-kernel $(root_kernel) \
		-append "root=/dev/vda mem=1536m" \
		-drive if=none,file=$(FSIMG1),id=hd1,format=raw \
		-device virtio-blk-device,drive=hd1 \
		-netdev tap,id=net0,ifname=tap0,script=no,downscript=no \
		-device virtio-net-device,netdev=net0,mac=52:55:00:d1:55:01 \
		-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3