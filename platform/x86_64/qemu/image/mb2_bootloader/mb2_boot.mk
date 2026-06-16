mb2_boot_dir := $(image_dir)/mb2_bootloader
mb2_boot_out_dir := $(image_dir)/mb2_bootloader/out

mb2_boot_src := $(mb2_boot_dir)/mb2_boot.S
mb2_boot_lds := $(mb2_boot_dir)/mb2_boot.ld

mb2_boot_o := $(mb2_boot_out_dir)/mb2_boot.o
mb2_boot_elf := $(mb2_boot_out_dir)/mb2_boot.elf
mb2_boot_bin := $(mb2_boot_out_dir)/mb2_boot.bin

AS ?= as
LD ?= ld
OBJCOPY ?= objcopy

# TSS descriptor: base=0x8048000, limit=103, type=0x89 (32-bit available TSS)
MB2_TSS_DESC := 0x8000890480000067
MB2_STACK   := 0x804a000

mb2_boot_flags := --32 -msyntax=intel -mnaked-reg
mb2_boot_flags += --defsym MB2_STACK=$(MB2_STACK)
mb2_boot_flags += --defsym MB2_TSS_DESCRIPTOR=$(MB2_TSS_DESC)

mb2_boot: | $(mb2_boot_out_dir) $(mb2_boot_bin)

$(mb2_boot_out_dir):
	mkdir -p $@

$(mb2_boot_o): $(mb2_boot_src)
	$(AS) $(mb2_boot_flags) $< -o $@

$(mb2_boot_elf): $(mb2_boot_o) $(mb2_boot_lds)
	$(LD) -T$(mb2_boot_lds) $< -o $@

$(mb2_boot_bin): $(mb2_boot_elf)
	$(OBJCOPY) $< --strip-all -O binary $@

.PHONY: mb2_boot
