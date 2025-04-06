QEMU := /home/sora/qemu/build/qemu-system-x86_64
# /home/sora/qemu/build/qemu-system-x86_64

zone0_setup := $(image_dir)/kernel/setup.bin
zone0_vmlinux := $(image_dir)/kernel/vmlinux.bin
zone0_boot16 := $(image_dir)/bootloader/boot16.bin
zone0_initrd := $(image_dir)/virtdisk/initramfs.cpio.gz
zone0_rootfs := $(image_dir)/virtdisk/rootfs1.img

QEMU_ARGS := -machine q35,kernel-irqchip=split
QEMU_ARGS += -cpu host,+x2apic,+invtsc -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 4G
QEMU_ARGS += -nographic
QEMU_ARGS += -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on

QEMU_ARGS += -device ioh3420,id=pcie.1,chassis=1
QEMU_ARGS += -drive if=none,file="$(zone0_rootfs)",id=X10008000,format=raw
QEMU_ARGS += -device virtio-blk-pci,bus=pcie.1,drive=X10008000,disable-legacy=on,disable-modern=off,iommu_platform=on,ats=on # bus=pcie.1,
# QEMU_ARGS += --trace "virtio_*" --trace "virtqueue_*" --trace "vtd_dma*" --trace "iommu_*"

QEMU_ARGS += -kernel $(hvisor_elf)
QEMU_ARGS += -device loader,file="$(zone0_boot16)",addr=0x5008000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_setup)",addr=0x500d000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_vmlinux)",addr=0x5100000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_initrd)",addr=0x20000000,force-raw=on
QEMU_ARGS += -append "initrd_size=$(shell stat -c%s $(zone0_initrd))"

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
