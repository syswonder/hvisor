# SPDX-License-Identifier: MulanPSL-2.0
QEMU := qemu-system-x86_64

kernel_dir := $(image_dir)/kernel
zone0_boot := $(image_dir)/bootloader/out/boot.bin
zone0_setup := $(kernel_dir)/asterinas-setup.bin
zone0_vmlinux := $(kernel_dir)/asterinas-vmlinux.bin
zone0_initrd := $(kernel_dir)/initramfs.cpio.gz

QEMU_ARGS := -machine q35,kernel-irqchip=split
QEMU_ARGS += -cpu host,+x2apic,+invtsc,+vmx -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -m 4G
QEMU_ARGS += -bios /usr/share/ovmf/OVMF.fd
QEMU_ARGS += -display none -serial mon:stdio
QEMU_ARGS += -nodefaults
QEMU_ARGS += -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48
QEMU_ARGS += -drive file=$(image_dir)/virtdisk/hvisor.iso,format=raw,index=0,media=disk

$(hvisor_bin): elf boot
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
	cp $(hvisor_elf) $(image_dir)/iso/boot
	mkdir -p $(image_dir)/iso/boot/kernel

	cp $(zone0_boot) $(image_dir)/iso/boot/kernel

	@for f in $(zone0_setup) $(zone0_vmlinux) $(zone0_initrd); do \
		if [ -f $$f ]; then \
			cp $$f $(image_dir)/iso/boot/kernel; \
		else \
			echo "Error: $$f not found; build Asterinas first (see platform/x86_64/qemu-asterinas/README.md)"; \
			exit 1; \
		fi; \
	done

	@# The guest reads the decompressed initramfs from a fixed window
	@# (ROOT_ZONE_INITRD_SIZE in board.rs); a larger archive would be truncated.
	@initrd_max=3145728; \
	 if ! gzip -t $(zone0_initrd) 2>/dev/null; then \
		echo "Error: $(zone0_initrd) is not a valid gzip archive"; \
		exit 1; \
	 fi; \
	 initrd_size=$$(zcat $(zone0_initrd) | wc -c); \
	 if [ $$initrd_size -gt $$initrd_max ]; then \
		echo "Error: decompressed initramfs is $$initrd_size bytes, exceeds the $$initrd_max-byte initrd window (board.rs ROOT_ZONE_INITRD_SIZE)"; \
		exit 1; \
	 fi

	mkdir -p $(image_dir)/virtdisk

	@if ! command -v xorriso >/dev/null 2>&1 || ! command -v grub-mkrescue >/dev/null 2>&1; then \
		echo "Error: xorriso and grub-mkrescue are required to build the ISO"; \
		exit 1; \
	fi
	@if [ ! -d /usr/lib/grub/x86_64-efi ]; then \
		echo "Error: /usr/lib/grub/x86_64-efi not found (install grub-efi-amd64-bin)"; \
		exit 1; \
	fi
	grub-mkrescue /usr/lib/grub/x86_64-efi -o $(image_dir)/virtdisk/hvisor.iso $(image_dir)/iso

include $(image_dir)/bootloader/boot.mk
