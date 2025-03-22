QEMU := qemu-system-x86_64

acpi_asl_dir := scripts/x86_64/acpi
acpi_aml_dir := $(image_dir)/acpi

zone0_bios := $(image_dir)/rvm-bios.bin
zone0_kernel := $(image_dir)/nimbos.bin

zone0_image := $(image_dir)/bzImage
zone0_setup := $(image_dir)/setup.bin
zone0_vmlinux := $(image_dir)/vmlinux.bin
zone0_initrd := $(image_dir)/initramfs.cpio.gz
zone0_boot16 := $(image_dir)/boot16.bin

aml_hpet := $(acpi_aml_dir)/hpet.aml
aml_madt := $(acpi_aml_dir)/madt.aml
aml_rsdp := $(acpi_aml_dir)/rsdp.aml
aml_rsdt := $(acpi_aml_dir)/rsdt.aml
aml_xsdt := $(acpi_aml_dir)/xsdt.aml

QEMU_ARGS := -machine q35
QEMU_ARGS += -cpu host,+x2apic -accel kvm
QEMU_ARGS += -smp 4
QEMU_ARGS += -serial mon:stdio
QEMU_ARGS += -m 2G
QEMU_ARGS += -nographic

QEMU_ARGS += -kernel $(hvisor_elf)
# QEMU_ARGS += -device loader,file="$(zone0_bios)",addr=0x5008000,force-raw=on
# QEMU_ARGS += -device loader,file="$(zone0_kernel)",addr=0x5200000,force-raw=on

QEMU_ARGS += -device loader,file="$(zone0_boot16)",addr=0x5008000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_setup)",addr=0x500d000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_vmlinux)",addr=0x5100000,force-raw=on
QEMU_ARGS += -device loader,file="$(zone0_initrd)",addr=0x20000000,force-raw=on
QEMU_ARGS += -append "initrd_size=$(shell stat -c%s $(zone0_initrd))"

QEMU_ARGS += -device loader,file="$(aml_rsdp)",addr=0x50f2400,force-raw=on
QEMU_ARGS += -device loader,file="$(aml_rsdt)",addr=0x50f2440,force-raw=on
QEMU_ARGS += -device loader,file="$(aml_xsdt)",addr=0x50f2480,force-raw=on
QEMU_ARGS += -device loader,file="$(aml_madt)",addr=0x50f2500,force-raw=on
QEMU_ARGS += -device loader,file="$(aml_hpet)",addr=0x50f2740,force-raw=on

$(hvisor_bin): elf aml
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@

aml: $(aml_hpet) $(aml_madt) $(aml_rsdp) $(aml_rsdt) $(aml_xsdt)

$(acpi_aml_dir)/%.aml: $(acpi_asl_dir)/%.asl
	iasl -p $@ $<