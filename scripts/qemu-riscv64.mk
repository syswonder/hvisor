QEMU := qemu-system-riscv64


FSIMG1 := $(image_dir)/virtdisk/rootfs1.ext4
# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x80200000
zone0_kernel := $(image_dir)/kernel/Image-62
zone0_dtb    := $(image_dir)/devicetree/linux3.dtb

QEMU_ARGS := -machine virt
QEMU_ARGS += -bios default
QEMU_ARGS += -cpu rv64
QEMU_ARGS += -smp 4
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic

QEMU_ARGS += -kernel $(hvisor_bin)
QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x90000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_dtb)",addr=0x84000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_kernel)",addr=0x70000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_dtb)",addr=0x91000000,force-raw=on

QEMU_ARGS += -drive if=none,file=$(FSIMG1),id=Xa003e000,format=raw
QEMU_ARGS += -device virtio-blk-device,drive=Xa003e000

# QEMU_ARGS += -drive if=none,file=$(FSIMG2),id=Xa003c000,format=raw
# QEMU_ARGS += -device virtio-blk-device,drive=Xa003c000

# QEMU_ARGS += -netdev tap,id=Xa003a000,ifname=tap0,script=no,downscript=no
# QEMU_ARGS += -device virtio-net-device,netdev=Xa003a000,mac=52:55:00:d1:55:01
# QEMU_ARGS += -netdev user,id=n0,hostfwd=tcp::5555-:22 -device virtio-net-device,bus=virtio-mmio-bus.29,netdev=n0 

# QEMU_ARGS += -chardev pty,id=Xa0038000
# QEMU_ARGS += -device virtio-serial-device,bus=virtio-mmio-bus.28 -device virtconsole,chardev=Xa0038000

# QEMU_ARGS += --fsdev local,id=Xa0036000,path=./9p/,security_model=none
# QEMU_ARGS += -device virtio-9p-pci,fsdev=Xa0036000,mount_tag=kmod_mount

# trace-event gicv3_icc_generate_sgi on
# trace-event gicv3_redist_send_sgi on
# QEMU_ARGS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
# QEMU_ARGS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
# QEMU_ARGS	+=-drive file=../guests/rCore-Tutorial-v3/fs.img,if=none,format=raw,id=x0
# QEMU_ARGS	+=-device virtio-blk-device,drive=x0
# QEMU_ARGS	+=-device virtio-gpu-device
# QEMU_ARGS	+=-device virtio-keyboard-device
# QEMU_ARGS	+=-device virtio-mouse-device
# QEMU_ARGS 	+=-device virtio-net-device,netdev=net0
# QEMU_ARGS	+=-netdev user,id=net0,hostfwd=udp::6200-:2000

# -------------------------------------------------------------------

# QEMU_ARGS := -machine virt
# QEMU_ARGS += -nographic 
# QEMU_ARGS += -cpu rv64 
# QEMU_ARGS += -m 3G 
# QEMU_ARGS += -smp 4 
# QEMU_ARGS += -bios default
# # QEMU_ARGS +=-bios $(BOOTLOADER)
# QEMU_ARGS += -kernel tenants/Image-62
# QEMU_ARGS += -drive file=imgs/rootfs-busybox.qcow2,format=qcow2,id=hd0 
# #QEMU_ARGS +=-drive file=../guests/rootfs-buildroot.qcow2,format=qcow2,id=hd0 
# QEMU_ARGS += -device virtio-blk-device,drive=hd0 
# QEMU_ARGS += -append "root=/dev/vda rw console=ttyS0"

# #QEMU_ARGS +=		 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) 
# #QEMU_ARGS +=		 -device loader,file=../guests/os_ch5_802.bin,addr=0x80400000 			 
# #QEMU_ARGS +=		 -device virtio-serial-port -chardev pty,id=serial3 -device virtconsole,chardev=serial3
# QEMU_ARGS +=		 -device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@