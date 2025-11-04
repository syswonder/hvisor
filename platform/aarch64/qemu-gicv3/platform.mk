QEMU := qemu-system-aarch64

UBOOT := $(image_dir)/bootloader/u-boot-atf.bin
zone0_dtb := $(image_dir)/dts/zone0.dtb
QEMU_ARGS := -machine virt,secure=on,gic-version=3,virtualization=on,iommu=smmuv3,acpi=on,nvdimm=on,mpam=on,mpam_min_msc=on
MESSAGE := "Note: Feature contains gicv3"

FSIMG1 := $(image_dir)/virtdisk/rootfs1.ext4
FSIMG2 := $(image_dir)/virtdisk/rootfs2.ext4

zone0_kernel := $(image_dir)/kernel/Image

QEMU_ARGS += -global arm-smmuv3.stage=2
QEMU_ARGS += -cpu cortex-a72,core-count=1
QEMU_ARGS += -smp 4,sockets=1,clusters=4,cores=1,threads=1,cache-cluster-start-level=2,cache-node-start-level=3
QEMU_ARGS += -m 4G
QEMU_ARGS += -nographic
QEMU_ARGS += -bios $(UBOOT)

QEMU_ARGS += -device loader,file="$(hvisor_bin)",addr=0x40400000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0xa0400000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_dtb)",addr=0xa0000000,force-raw=on

QEMU_ARGS += -drive if=none,file=$(FSIMG1),id=Xa003e000,format=raw
QEMU_ARGS += -device virtio-blk-device,drive=Xa003e000,bus=virtio-mmio-bus.31

$(hvisor_bin): elf
	@if ! command -v mkimage > /dev/null; then \
		sudo apt update && sudo apt install u-boot-tools; \
	fi && \
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp && \
	mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
	-e 0x40400000 -d $(hvisor_bin).tmp $(hvisor_bin)

QEMU_ARGS += -netdev type=user,id=net1
QEMU_ARGS += -device virtio-net-pci,netdev=net1,disable-legacy=on,disable-modern=off,iommu_platform=on

# QEMU_ARGS += -device pci-testdev

QEMU_ARGS += -netdev type=user,id=net2
QEMU_ARGS += -device virtio-net-pci,netdev=net2,disable-legacy=on,disable-modern=off,iommu_platform=on

QEMU_ARGS += -netdev type=user,id=net3
QEMU_ARGS += -device virtio-net-pci,netdev=net3,disable-legacy=on,disable-modern=off,iommu_platform=on

# NUMA configuration
# Socket 0
# ├─ Cluster 0 (NUMA Node 0)
# │  ├─ vCPU 0
# │  └─ Memory: 1GB
# │
# ├─ Cluster 1 (NUMA Node 1)
# │  ├─ vCPU 1
# │  └─ Memory: 1GB
# │
# ├─ Cluster 2 (NUMA Node 2)
# │  ├─ vCPU 2
# │  └─ Memory: 1GB
# │
# └─ Cluster 3 (NUMA Node 3)
#    ├─ vCPU 3
#    └─ Memory: 1GB

QEMU_ARGS += -object memory-backend-ram,size=1G,id=mem0 
QEMU_ARGS += -object memory-backend-ram,size=1G,id=mem1 
QEMU_ARGS += -object memory-backend-ram,size=1G,id=mem2
QEMU_ARGS += -object memory-backend-ram,size=1G,id=mem3
QEMU_ARGS += -numa node,nodeid=0,cpus=0,memdev=mem0
QEMU_ARGS += -numa node,nodeid=1,cpus=1,memdev=mem1
QEMU_ARGS += -numa node,nodeid=2,cpus=2,memdev=mem2
QEMU_ARGS += -numa node,nodeid=3,cpus=3,memdev=mem3