QEMU := qemu-system-x86_64

zone0_bios := $(image_dir)/rvm-bios.bin
zone0_kernel := $(image_dir)/nimbos.bin

QEMU_ARGS := -machine q35
QEMU_ARGS += -cpu host -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic

QEMU_ARGS += -kernel $(hvisor_elf)
QEMU_ARGS += -device loader,file="$(zone0_bios)",addr=0x1008000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x1200000,force-raw=on

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@