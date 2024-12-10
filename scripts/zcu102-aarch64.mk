# Makefile for Xilinx ZCU102 AArch64 platform
# created on 2024.12.2, wheatfox(enkerewpo@hotmail.com)

# according to petalinux-boot qemu
# however we must use petalinux-boot qemu to use it because it need some background servers

# qemu-system-aarch64 -M arm-generic-fdt
# -serial mon:stdio -serial /dev/null -display none 
# -device loader,file=system.dtb,addr=0x100000,force-raw=on
# -device loader,file=u-boot.elf
# -device loader,file=Image,addr=0x200000,force-raw=on
# -device loader,file=rootfs.cpio.gz.u-boot,addr=0x4000000,force-raw=on
# -device loader,file=bl31.elf,cpu-num=0
# -global xlnx,zynqmp-boot.cpu-num=0
# -global xlnx,zynqmp-boot.use-pmufw=true
# -global xlnx,zynqmp-boot.drive=pmu-cfg
# -blockdev node-name=pmu-cfg,filename=pmu-conf.bin,driver=file

# -hw-dtb zynqmp-qemu-multiarch-arm.dtb # <- this argument is added by xilinx for their qemu port

# -device loader,file=boot.scr,addr=0x20000000,force-raw=on
# -gdb tcp:localhost:9000
# -net nic -net nic -net nic -net nic,netdev=eth3 -netdev user,id=eth3,tftp=/tftpboot
# -machine-path /tmp/tmpbf8tgt6q
# -m 4G

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
	-e 0x40400000 -d $(hvisor_bin).tmp $(hvisor_bin) && \
	rm -rf $(hvisor_bin).tmp


HVISOR_BIN_FULL_PATH = $(shell readlink -f $(hvisor_bin))
DEFAULT_PETALINUX_PROJECT_PATH = /home/wheatfox/Documents/Code/petalinux_projects/wheatfox_hw0
DEFAULT_PETALINUX_SDK_PATH = /home/wheatfox/petalinux_sdk

FAKE_DTB = $(PETALINUX_PROJECT_PATH)/images/linux/system.dtb
FAKE_DTB_PATH = $(shell readlink -f $(FAKE_DTB))

# args passed to xilinx's qemu
EXTRA_QEMU_ARGS =
EXTRA_QEMU_ARGS += -device loader,file=$(HVISOR_BIN_FULL_PATH),addr=0x40400000,force-raw=on
EXTRA_QEMU_ARGS += -device loader,file=$(FAKE_DTB_PATH),addr=0x100000,force-raw=on

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

.PHONY: run-petalinux-qemu
run-petalinux-qemu: $(hvisor_bin)
	@echo "Running petalinux qemu..."
# petalinux only works in bash
# it will open a gdb server on tcp:localhost:9000
	bash -c "source $(PETALINUX_SDK_ROOT)/settings.sh && \
		cd $(PETALINUX_PROJECT_PATH) && petalinux-boot qemu \
		--prebuilt 2 \
		--qemu-args '$(EXTRA_QEMU_ARGS)' \
		"

GDB ?= aarch64-linux-gnu-gdb

.PHONY: debug-petalinux-qemu
debug-petalinux-qemu:
	@echo "Starting gdb client..."
	$(GDB) -ex "target remote localhost:9000" -ex "layout asm"

# manual uboot boot: bootm 0x40400000 - 0x100000