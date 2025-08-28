
# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x80200000
BOOT_PATH := $(image_dir)/../


$(hvisor_bin): elf
	@if ! command -v mkimage > /dev/null; then \
		sudo apt update && sudo apt install u-boot-tools; \
	fi && \
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp && \
	mkimage -n hvisor_img -A riscv -O linux -C none -T kernel -a $(HVISOR_ENTRY_PA) \
	-e $(HVISOR_ENTRY_PA) -d $(hvisor_bin).tmp $(hvisor_bin) && \
	rm -rf $(hvisor_bin).tmp

mkscr:
	mkimage -A arm -T script -C none -n "Boot Script" -d $(BOOT_PATH)boot.txt $(BOOT_PATH)boot.scr