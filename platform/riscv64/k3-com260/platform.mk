
BOOT_PATH        := $(image_dir)/../
HVISOR_RAW_BIN_ABS := $(abspath $(hvisor_bin).tmp)

$(hvisor_bin): elf
	@if ! command -v mkimage > /dev/null; then \
		sudo apt-get install -y u-boot-tools; \
	fi
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp
	cp $(BOOT_PATH)hvisor.its $(BOOT_PATH)hvisor.its.tmp
	sed -i 's|hvisor.bin.tmp|$(HVISOR_RAW_BIN_ABS)|g' $(BOOT_PATH)hvisor.its.tmp
	mkimage -f $(BOOT_PATH)hvisor.its.tmp $(hvisor_bin)
	rm -f $(hvisor_bin).tmp $(BOOT_PATH)hvisor.its.tmp
