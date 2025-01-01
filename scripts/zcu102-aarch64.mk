# Makefile for Xilinx ZCU102 AArch64 platform
# created on 2024.12.2, wheatfox(enkerewpo@hotmail.com)

# according to petalinux-boot qemu
# however we must use petalinux-boot qemu to use it because it need some background servers ?

$(hvisor_bin): elf
	@if ! command -v mkimage > /dev/null; then \
		if [ "$(shell uname)" = "Linux" ]; then \
			echo "mkimage not found. Installing using apt..."; \
			sudo apt update && sudo apt install -y u-boot-tools; \
		elif [ "$(shell uname)" = "Darwin" ]; then \
			echo "mkimage not found. Installing using brew, you may need to reopen the Terminal App"; \
			brew install u-boot-tools; \
		else \
			echo "Unsupported operating system. Please install u-boot-tools manually."; \
			exit 1; \
		fi; \
	fi && \
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp && \
	mkimage -n hvisor_img -A arm64 -O linux -C none -T kernel -a 0x40400000 \
	-e 0x40400000 -d $(hvisor_bin).tmp $(hvisor_bin)

HVISOR_BIN_FULL_PATH = $(shell readlink -f $(hvisor_bin))
DEFAULT_PETALINUX_PROJECT_PATH = /home/wheatfox/Documents/Code/petalinux_projects/wheatfox_hw0
DEFAULT_PETALINUX_SDK_PATH = /home/wheatfox/petalinux_sdk

ROOT_LINUX_IMAGE = $(PETALINUX_PROJECT_PATH)/images/linux/vmlinux
ROOT_LINUX_IMAGE_BIN = $(ROOT_LINUX_IMAGE).bin
ROOT_LINUX_ROOTFS = $(PETALINUX_PROJECT_PATH)/images/linux/rootfs.cpio.gz.u-boot
# ROOT_LINUX_DTB = $(PETALINUX_PROJECT_PATH)/images/linux/system.dtb
ROOT_LINUX_DTB = $(shell readlink -f ./images/aarch64/devicetree/zcu102-root-aarch64.dtb)
ROOT_LINUX_SD_IMG = $(shell readlink -f ./sd.img)

# notes on uboot FIT:
# please pass the raw vmlinux and hvisor stripped binary in uboot's its
# because when generating FIT image, uboot will create the "Image" format binary and embed it in the FIT image
# so, don't pass the "Image" format binary or hvisor.bin to uboot's its because
# this will cause double wrapping of the binary ! - wheatfox
TARGET_FIT_IMAGE = fitImage
TARGET_FIT_IMAGE_PATH = $(shell readlink -f $(TARGET_FIT_IMAGE))

GDB ?= aarch64-linux-gnu-gdb
READELF ?= aarch64-linux-gnu-readelf
OBJDUMP = aarch64-linux-gnu-objdump

ifndef PETALINUX_SDK_ROOT
	PETALINUX_SDK_ROOT = $(DEFAULT_PETALINUX_SDK_PATH)
else
# set PETALINUX_SDK_ROOT in your environment to override the default
	PETALINUX_SDK_ROOT = $(PETALINUX_SDK_ROOT)	
endif


ifndef PETALINUX_PROJECT_PATH
	PETALINUX_PROJECT_PATH = $(DEFAULT_PETALINUX_PROJECT_PATH)
else
# set PETALINUX_PROJECT_PATH in your environment to override the default
	PETALINUX_PROJECT_PATH = $(PETALINUX_PROJECT_PATH)
endif

HVISOR_TMP_PATH = $(shell readlink -f $(hvisor_bin).tmp)
GCC_OBJCOPY = aarch64-linux-gnu-objcopy

.PHONY: dtb
dtb:
	make -C ./images/aarch64/devicetree

.PHONY: gen-fit
gen-fit: $(hvisor_bin) dtb
	@if [ ! -f scripts/zcu102-aarch64-fit.its ]; then \
		echo "Error: ITS file scripts/zcu102-aarch64-fit.its not found."; \
		exit 1; \
	fi
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(HVISOR_TMP_PATH)
# now we need to create the vmlinux.bin
	$(GCC_OBJCOPY) $(ROOT_LINUX_IMAGE) --strip-all -O binary $(ROOT_LINUX_IMAGE_BIN)
	@sed \
		-e "s|__ROOT_LINUX_IMAGE__|$(ROOT_LINUX_IMAGE_BIN)|g" \
		-e "s|__ROOT_LINUX_ROOTFS__|$(ROOT_LINUX_ROOTFS)|g" \
		-e "s|__ROOT_LINUX_DTB__|$(ROOT_LINUX_DTB)|g" \
		-e "s|__HVISOR_TMP_PATH__|$(HVISOR_TMP_PATH)|g" \
		scripts/zcu102-aarch64-fit.its > temp-fit.its
	@mkimage -f temp-fit.its $(TARGET_FIT_IMAGE)
	@echo "Generated FIT image: $(TARGET_FIT_IMAGE)"

# "pl" is short for "petalinux"
# args passed to xilinx's qemu
EXTRA_QEMU_ARGS =
EXTRA_QEMU_ARGS += -device loader,file=$(TARGET_FIT_IMAGE_PATH),addr=0x10000000,force-raw=on
# add SD
EXTRA_QEMU_ARGS += -drive file=$(ROOT_LINUX_SD_IMG),format=raw,if=sd

.PHONY: run-pl-qemu
run-pl-qemu: $(hvisor_bin) gen-fit
	@echo "Running petalinux qemu..."
# petalinux only works in bash
# it will open a gdb server on tcp:localhost:9000
	bash -c "source $(PETALINUX_SDK_ROOT)/settings.sh && \
		cd $(PETALINUX_PROJECT_PATH) && petalinux-boot qemu \
		--prebuilt 2 \
		--qemu-args '$(EXTRA_QEMU_ARGS)' \
		"

# uboot cmds:\
setenv fit_addr 0x10000000;setenv root_linux_load 0x200000;setenv root_rootfs_load 0x4000000;imxtract ${fit_addr} root_linux ${root_linux_load};imxtract ${fit_addr} root_rootfs ${root_rootfs_load};md ${root_linux_load} 20;bootm ${fit_addr};

.PHONY: debug-pl-qemu
debug-pl-qemu:
	@echo "Starting gdb client..."
	$(GDB) -ex "target remote localhost:9000" -ex "layout asm"

# below are some quick commands
.PHONY: pl-build
pl-build:
	@echo "Building petalinux project..."
	bash -c "source $(PETALINUX_SDK_ROOT)/settings.sh && \
		cd $(PETALINUX_PROJECT_PATH) && petalinux-build"

.PHONY: vmlinux-info
vmlinux-info:
	@$(READELF) -h $(ROOT_LINUX_IMAGE) | grep "Entry point address" | awk '{print $$0}'
	@$(READELF) -l $(ROOT_LINUX_IMAGE) | awk '{print $$0}'

.PHONY: disasm-vmlinux
disasm-vmlinux:
	@$(OBJDUMP) -d $(ROOT_LINUX_IMAGE) > target/petalinux-vmlinux-disasm.txt

.PHONY: pl-config
pl-config:
	@echo "Configuring petalinux project..."
	bash -c "source $(PETALINUX_SDK_ROOT)/settings.sh && \
		cd $(PETALINUX_PROJECT_PATH) && petalinux-config"

.PHONY: sd-image
# you can customized the real rootfs by changing the content of this sd.img
sd-image:
	@echo "Creating SD card image..."
	qemu-img create -f raw sd.img 1G