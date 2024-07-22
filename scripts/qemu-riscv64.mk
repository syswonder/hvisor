QEMU := qemu-system-riscv64

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

FSIMG1 := $(image_dir)/virtdisk/rootfs1.ext4
FSIMG2 := $(image_dir)/virtdisk/rootfs-busybox.qcow2
# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x80200000
# plic
zone0_kernel := $(image_dir)/kernel/Image-aia-6.10
zone0_dtb    := $(image_dir)/devicetree/linux1.dtb
# zone1_kernel := $(image_dir)/kernel/Image
# zone1_dtb    := $(image_dir)/devicetree/linux2.dtb

# aia
zone0_aia_kernel := $(image_dir)/kernel/Image-aia-6.10
zone0_aia_dtb    := $(image_dir)/devicetree/linux1-aia.dtb
# zone1_aia_kernel := $(image_dir)/kernel/Image-aia
# zone1_aia_dtb    := $(image_dir)/devicetree/linux2-aia.dtb

ifeq ($(IRQ),aia)
    QEMU_ARGS := -machine virt,aclint=on,aia=aplic-imsic,aia-guests=1
else ifeq ($(IRQ),plic)
    QEMU_ARGS := -machine virt
endif

QEMU_ARGS += -bios $(bios_elf)
QEMU_ARGS += -cpu rv64
QEMU_ARGS += -smp 4
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic

QEMU_ARGS +=		 -device loader,file=$(hvisor_bin),addr=$(HVISOR_ENTRY_PA) 

ifeq ($(IRQ),aia)
QEMU_ARGS += -device loader,file="$(zone0_aia_kernel)",addr=0x90000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_aia_dtb)",addr=0x8f000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_aia_kernel)",addr=0x84000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_aia_dtb)",addr=0x83000000,force-raw=on
else ifeq ($(IRQ),plic)
QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x90000000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_dtb)",addr=0x8f000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_kernel)",addr=0x84000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone1_dtb)",addr=0x83000000,force-raw=on
endif

QEMU_ARGS += -drive if=none,file=$(FSIMG1),id=X10008000,format=raw
QEMU_ARGS += -device virtio-blk-device,drive=X10008000
QEMU_ARGS += -device virtio-serial-device -chardev pty,id=X10007000 -device virtconsole,chardev=X10007000 -S
QEMU_ARGS += -drive if=none,file=$(FSIMG2),id=X10006000,format=qcow2
QEMU_ARGS += -device virtio-blk-device,drive=X10006000
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

