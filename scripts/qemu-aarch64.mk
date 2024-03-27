QEMU := echo -e "\nbootm 0x40400000 - 0x40000000" | qemu-system-aarch64

zone0_kernel := tenants/aarch64/kernel/Image
zone0_dtb    := tenants/aarch64/devicetree/linux1.dtb
zone1_kernel := tenants/aarch64/kernel/Image
zone1_dtb    := tenants/aarch64/devicetree/linux2.dtb

QEMU_ARGS := -machine virt,secure=on,gic-version=3,virtualization=on
QEMU_ARGS += -cpu cortex-a57
QEMU_ARGS += -smp 4
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic
QEMU_ARGS += -bios imgs/u-boot.bin

QEMU_ARGS += -device loader,file="$(hvisor_bin)",addr=0x40400000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x50000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_dtb)",addr=0x90000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone1_kernel)",addr=0x70000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone1_dtb)",addr=0x91000000,force-raw=on

# -drive if=none,file=$(FSIMG1),id=Xa003e000,format=raw \
# -device virtio-blk-device,drive=Xa003e000 \
# \
# -drive if=none,file=$(FSIMG2),id=Xa003c000,format=raw \
# -device virtio-blk-device,drive=Xa003c000 \
# \
# -netdev tap,id=Xa003a000,ifname=tap0,script=no,downscript=no \
# -device virtio-net-device,netdev=Xa003a000,mac=52:55:00:d1:55:01 \
# \
# -chardev pty,id=Xa0038000 \
# -device virtio-serial-device -device virtconsole,chardev=Xa0038000

# FSIMG1=/path/to/disk1.img
# FSIMG2=/path/to/disk2.img
 
# # QEMU command template
# define qemu_cmd
# e2fsck -f $(FSIMG1) && \
# e2fsck -f $(FSIMG2) && \
# sudo qemu-system-aarch64 \
# 	-machine virt,secure=on,gic-version=3,virtualization=on \
# 	-cpu cortex-a57 \
# 	-smp 4 \
# 	-m 2G \
# 	-nographic \
# 	-bios imgs/u-boot.bin \
# 	\
# 	-device loader,file="$(hvisor_bin)",addr=0x7fc00000,force-raw=on \
# 	-device loader,file="$(root_dtb)",addr=0x40100000,force-raw=on \
# 	-device loader,file="$(root_kernel)",addr=0x40200000,force-raw=on \
# 	-device loader,file="$(nr1_dtb)",addr=0x60100000,force-raw=on \
# 	-device loader,file="$(nr1_kernel)",addr=0x60200000,force-raw=on \
# 	\
# 	-drive if=none,file=$(FSIMG1),id=Xa003e000,format=raw \
# 	-device virtio-blk-device,drive=Xa003e000 \
# 	\
# 	-drive if=none,file=$(FSIMG2),id=Xa003c000,format=raw \
# 	-device virtio-blk-device,drive=Xa003c000 \
# 	\
# 	-netdev tap,id=Xa003a000,ifname=tap0,script=no,downscript=no \
# 	-device virtio-net-device,netdev=Xa003a000,mac=52:55:00:d1:55:01 \
# 	\
# 	-chardev pty,id=Xa0038000 \
# 	-device virtio-serial-device -device virtconsole,chardev=Xa0038000
# endef

# baremetal:
# 	sudo qemu-system-aarch64 \
# 		-machine virt,secure=on,gic-version=3,virtualization=on \
# 		-cpu cortex-a57 \
# 		-smp 4 \
# 		-m 2G \
# 		-nographic \
# 		-kernel $(root_kernel) \
# 		-append "root=/dev/vda mem=1536m" \
# 		-drive if=none,file=$(FSIMG1),id=hd1,format=raw \
# 		-device virtio-blk-device,drive=hd1 \
# 		-netdev tap,id=net0,ifname=tap0,script=no,downscript=no \
# 		-device virtio-net-device,netdev=net0,mac=52:55:00:d1:55:01 \
# 		-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3

$(hvisor_bin): elf
	@if ! command -v mkimage > /dev/null; then \
		sudo apt update && sudo apt install u-boot-tools; \
	fi && \
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp && \
	mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
	-e 0x40400000 -d $(hvisor_bin).tmp $(hvisor_bin) && \
	rm -rf $(hvisor_bin).tmp