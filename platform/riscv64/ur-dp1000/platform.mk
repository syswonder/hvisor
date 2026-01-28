
# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x80200000

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
