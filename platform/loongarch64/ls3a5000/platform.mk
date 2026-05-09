# hvisor for loongarch64 makefile
# wheatfox(wheatfox17@icloud.com) 2024.6
# boneinscri(boneinscri@163.com) 2026.4

# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x9000000080000000

# zone0_kernel := $(image_dir)/kernel/Image
# zone0_dtb    := $(image_dir)/devicetree/linux.dtb

# QEMU for loongarch64 doesn't support LVZ extension yet
# so no qemu related stuff here, we have to debug it on 
# REAL hardware with UEFI firmware interface

# QEMU := qemu-system-loongarch64
# QEMU_ARGS := -machine virt
# QEMU_ARGS += -bios default
# QEMU_ARGS += -smp 4
# QEMU_ARGS += -m 2G
# QEMU_ARGS += -nographic
# QEMU_ARGS += -kernel $(hvisor_bin)
# QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x90000000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_dtb)",addr=0x8f000000,force-raw=on

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
# objdump + hvisor-trap-vector.txt
	readelf -a $(hvisor_elf) > hvisor-elf.txt
	loongarch64-linux-gnu-objdump --disassemble $(hvisor_elf) > hvisor.S
	cp $(hvisor_elf) hvisor.elf
	nm -n hvisor.elf | grep -w _hyp_trap_vector | awk '{print "0x"$$1""}' > hvisor-trap-vector.txt