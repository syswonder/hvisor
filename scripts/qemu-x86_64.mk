QEMU := qemu-system-x86_64

QEMU_ARGS := -machine q35
QEMU_ARGS += -cpu host -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic
QEMU_ARGS += -kernel $(hvisor_elf)

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@