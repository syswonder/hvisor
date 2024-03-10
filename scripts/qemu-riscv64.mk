QEMU := qemu-system-riscv64

QEMU_ARGS := -machine virt 
QEMU_ARGS +=-nographic 
QEMU_ARGS +=-cpu rv64 
QEMU_ARGS +=-m 3G 
QEMU_ARGS +=-bios default
# QEMU_ARGS +=-bios $(BOOTLOADER)
QEMU_ARGS +=-kernel $(target_bin)
QEMU_ARGS +=-drive file=imgs/rootfs-busybox.qcow2,format=qcow2,id=hd0 
#QEMU_ARGS +=-drive file=../guests/rootfs-buildroot.qcow2,format=qcow2,id=hd0 
QEMU_ARGS +=-device virtio-blk-device,drive=hd0 
QEMU_ARGS +=-append "root=/dev/vda rw console=ttyS0"

#QEMU_ARGS +=		 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) 
#QEMU_ARGS +=		 -device loader,file=../guests/os_ch5_802.bin,addr=0x80400000 			 
#QEMU_ARGS +=		 -device virtio-serial-port -chardev pty,id=serial3 -device virtconsole,chardev=serial3 \


# QEMU_ARGS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
# QEMU_ARGS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
# QEMU_ARGS	+=-drive file=../guests/rCore-Tutorial-v3/fs.img,if=none,format=raw,id=x0
# QEMU_ARGS	+=-device virtio-blk-device,drive=x0
# QEMU_ARGS	+=-device virtio-gpu-device
# QEMU_ARGS	+=-device virtio-keyboard-device
# QEMU_ARGS	+=-device virtio-mouse-device
# QEMU_ARGS 	+=-device virtio-net-device,netdev=net0
# QEMU_ARGS	+=-netdev user,id=net0,hostfwd=udp::6200-:2000
