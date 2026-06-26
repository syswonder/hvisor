QEMU := qemu-system-x86_64
QEMU_PREFIX := $(shell qemu_path="$$(command -v $(QEMU) 2>/dev/null)"; if [ -n "$$qemu_path" ]; then cd "$$(dirname "$$qemu_path")/.." && pwd; fi)
OVMF ?= $(firstword $(wildcard $(QEMU_PREFIX)/share/ovmf/OVMF.fd $(QEMU_PREFIX)/share/OVMF/OVMF.fd $(QEMU_PREFIX)/share/OVMF/OVMF_CODE.fd $(QEMU_PREFIX)/share/edk2/ovmf/OVMF_CODE.fd))

zone0_boot := $(image_dir)/bootloader/out/boot.bin
zone0_setup := $(image_dir)/kernel/setup.bin
zone0_vmlinux := $(image_dir)/kernel/vmlinux.bin
zone0_asterinas := $(image_dir)/kernel/aster-kernel-osdk-bin
zone0_initrd := $(image_dir)/virtdisk/initramfs.cpio.gz
zone0_rootfs := $(image_dir)/virtdisk/rootfs1.img
zone1_rootfs := $(image_dir)/virtdisk/rootfs2.img

QEMU_ARGS := -machine q35,kernel-irqchip=split
QEMU_ARGS += -cpu host,+x2apic,+invtsc,+vmx -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 4G
ifneq ($(OVMF),)
QEMU_ARGS += -bios $(OVMF)
endif
QEMU_ARGS += -vga std
# QEMU_ARGS += -nographic

QEMU_ARGS += -nodefaults
QEMU_ARGS += -net nic -net user

QEMU_ARGS += -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48
QEMU_ARGS += -device ioh3420,id=pcie.1,chassis=1
QEMU_ARGS += -drive if=none,file="$(zone0_rootfs)",id=X10008000,format=raw
QEMU_ARGS += -device virtio-blk-pci,bus=pcie.1,drive=X10008000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on

# QEMU_ARGS += -drive if=none,file="$(zone0_rootfs)",id=X10009000,format=raw
# QEMU_ARGS += -device nvme,serial=deadbeef,drive=X10009000
# QEMU_ARGS += -drive if=none,file="$(zone1_rootfs)",id=X10009000,format=raw
# QEMU_ARGS += -device virtio-blk-pci,bus=pcie.1,drive=X10009000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += -netdev tap,id=net0,ifname=tap0,script=no,downscript=no
# QEMU_ARGS += -device virtio-net-pci,bus=pcie.1,netdev=net0,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += -netdev tap,id=net0,vhostforce=on
# QEMU_ARGS += -device virtio-net-pci,bus=pcie.1,netdev=net0,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += --trace "virtio_*" --trace "virtqueue_*" --trace "vtd_dma*" --trace "iommu_*"

# QEMU_ARGS += -kernel $(hvisor_elf)
QEMU_ARGS += -drive file=$(image_dir)/virtdisk/hvisor.iso,format=raw,index=0,media=disk
QEMU_ARGS += -nographic


# QEMU_ARGS += -device loader,file="$(zone0_boot)",addr=0x5008000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_setup)",addr=0x500a000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_vmlinux)",addr=0x5100000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_initrd)",addr=0x1a000000,force-raw=on
# QEMU_ARGS += -append "initrd_size=$(shell stat -c%s $(zone0_initrd))"

iso_build := $(image_dir)/iso-build

$(hvisor_bin): elf boot
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
# Assemble the bootable image in a build directory so the tracked iso/ tree (only
# grub.cfg) is never mutated by a build.
	rm -rf $(iso_build)
	cp -r $(image_dir)/iso $(iso_build)
	cp $(hvisor_elf) $(iso_build)/boot
	mkdir -p $(iso_build)/boot/kernel $(image_dir)/virtdisk
	for f in $(zone0_boot) $(zone0_setup) $(zone0_vmlinux) $(zone0_asterinas) $(zone0_initrd); do \
		if [ -f $$f ]; then cp $$f $(iso_build)/boot/kernel; \
		else echo "Warning: $$f not found, skipping"; fi; \
	done
# Default to the Asterinas entry for an aster_guest build; the tracked grub.cfg
# keeps the Linux entry as its committed default.
	if echo "$(FEATURES)" | grep -qw aster_guest; then \
		sed -i 's/^set default=.*/set default=1   # Asterinas/' $(iso_build)/boot/grub/grub.cfg; \
	fi
	if [ -n "$(SKIP_ISO)" ]; then \
		echo "SKIP_ISO set: not creating the bootable ISO"; \
	elif command -v xorriso >/dev/null 2>&1; then \
		grub-mkrescue -o $(image_dir)/virtdisk/hvisor.iso $(iso_build); \
	else \
		echo "Error: xorriso/grub-mkrescue is required to build the ISO (set SKIP_ISO=1 to skip)" >&2; \
		exit 1; \
	fi

# Headless run target for the Asterinas root zone. Build the matching binary with
#   make ARCH=x86_64 BOARD=qemu FEATURES="<defaults> aster_guest" all
# then `make ... run-asterinas`. The guest console is on COM1 (mon:stdio); exit
# QEMU with Ctrl-A x.
ASTER_QEMU_ARGS := -machine q35,kernel-irqchip=split
ASTER_QEMU_ARGS += -cpu host,+x2apic,+invtsc,+vmx -accel kvm
ASTER_QEMU_ARGS += -smp 4 -m 4G
ifneq ($(OVMF),)
ASTER_QEMU_ARGS += -bios $(OVMF)
endif
ASTER_QEMU_ARGS += -nographic -serial mon:stdio -nodefaults
ASTER_QEMU_ARGS += -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48
ASTER_QEMU_ARGS += -device ioh3420,id=pcie.1,chassis=1
ASTER_QEMU_ARGS += -drive file=$(image_dir)/virtdisk/hvisor.iso,format=raw,index=0,media=disk

run-asterinas: all
	@test -n "$(OVMF)" || { echo "OVMF firmware not found; set OVMF=..."; exit 1; }
	$(QEMU) $(ASTER_QEMU_ARGS)

include $(image_dir)/bootloader/boot.mk
