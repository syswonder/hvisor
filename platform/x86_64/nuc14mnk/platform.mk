QEMU := qemu-system-x86_64

zone0_boot := $(image_dir)/bootloader/out/boot.bin
zone0_setup := $(image_dir)/kernel/setup.bin
zone0_vmlinux := $(image_dir)/kernel/vmlinux.bin
zone0_initrd := $(image_dir)/virtdisk/initramfs.cpio.gz
zone0_rootfs := $(image_dir)/virtdisk/rootfs1.img
zone1_rootfs := $(image_dir)/virtdisk/rootfs2.img

QEMU_ARGS := -machine q35,kernel-irqchip=split
QEMU_ARGS += -cpu host,+x2apic,+invtsc,+vmx -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 4G
QEMU_ARGS += -bios /usr/share/ovmf/OVMF.fd
QEMU_ARGS += -vga std
# QEMU_ARGS += -nographic

QEMU_ARGS += -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48
QEMU_ARGS += -device ioh3420,id=pcie.1,chassis=1
QEMU_ARGS += -drive if=none,file="$(zone0_rootfs)",id=X10008000,format=raw
QEMU_ARGS += -device virtio-blk-pci,bus=pcie.1,drive=X10008000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on

# QEMU_ARGS += -drive if=none,file="$(zone1_rootfs)",id=X10009000,format=raw
# QEMU_ARGS += -device virtio-blk-pci,bus=pcie.1,drive=X10009000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += -netdev tap,id=net0,ifname=tap0,script=no,downscript=no
# QEMU_ARGS += -device virtio-net-pci,bus=pcie.1,netdev=net0,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += -netdev tap,id=net0,vhostforce=on
# QEMU_ARGS += -device virtio-net-pci,bus=pcie.1,netdev=net0,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on
# QEMU_ARGS += --trace "virtio_*" --trace "virtqueue_*" --trace "vtd_dma*" --trace "iommu_*"

# QEMU_ARGS += -kernel $(hvisor_elf)
QEMU_ARGS += -drive file=$(image_dir)/virtdisk/hvisor.iso,format=raw,index=0,media=disk

# QEMU_ARGS += -device loader,file="$(zone0_boot)",addr=0x5008000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_setup)",addr=0x500a000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_vmlinux)",addr=0x5100000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_initrd)",addr=0x20000000,force-raw=on
# QEMU_ARGS += -append "initrd_size=$(shell stat -c%s $(zone0_initrd))"

$(hvisor_bin): elf boot
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
	cp $(hvisor_elf) $(image_dir)/iso/boot
	mkdir -p $(image_dir)/iso/boot/kernel
	cp $(zone0_boot) $(image_dir)/iso/boot/kernel
	cp $(zone0_setup) $(image_dir)/iso/boot/kernel
	cp $(zone0_vmlinux) $(image_dir)/iso/boot/kernel
	mkdir -p $(image_dir)/virtdisk
	grub-mkrescue /usr/lib/grub/x86_64-efi -o $(image_dir)/virtdisk/hvisor.iso $(image_dir)/iso

include $(image_dir)/bootloader/boot.mk