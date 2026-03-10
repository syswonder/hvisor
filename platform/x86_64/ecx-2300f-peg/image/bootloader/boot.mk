boot_dir := $(image_dir)/bootloader
boot_out_dir := $(image_dir)/bootloader/out

boot_src := $(boot_dir)/boot.S
boot_lds := $(boot_dir)/boot.ld

boot_o := $(boot_out_dir)/boot.o
boot_elf := $(boot_out_dir)/boot.elf
boot_bin := $(boot_out_dir)/boot.bin
boot_disa := $(boot_out_dir)/boot.asm

AS ?= as
LD ?= ld
OBJCOPY ?= objcopy
OBJDUMP ?= objdump

boot: mkout $(boot_bin)

disasm:
	$(OBJDUMP) -d -m i8086 -M intel $(boot_elf) | less

mkout:
	rm -rf $(boot_out_dir)
	mkdir -p $(boot_out_dir)

$(boot_o): $(boot_src)
	$(AS) --32 -msyntax=intel -mnaked-reg $< -o $@

$(boot_elf): $(boot_o) $(boot_lds)
	$(LD) -T$(boot_lds) $< -o $@
	$(OBJDUMP) -d -m i8086 -M intel $@ > $(boot_disa)

$(boot_bin): $(boot_elf)
	$(OBJCOPY) $< --strip-all -O binary $@

.PHONY: all disasm