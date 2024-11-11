KDIR ?= ../../nxp/OK8MP-linux-kernel
DEV ?= /dev/sda1
ARCH ?= arm64
LOG ?= LOG_WARN
DEBUG ?= n

export KDIR
export ARCH
export LOG
.PHONY: all env tools driver clean

all: tools driver

env:
	git submodule update --init --recursive

tools: env
	$(MAKE) -C tools all

driver: env
	$(MAKE) -C driver all

transfer: all
	./trans_file.sh ./tools/hvisor 
	./trans_file.sh ./driver/hvisor.ko 

# transfer_nxp: all
# 	sudo mount $(DEV) /mnt/
# 	sudo rm -f /mnt/home/arm64/hvisor /mnt/home/arm64/hvisor.ko
# 	sudo cp ./tools/hvisor /mnt/home/arm64
# 	sudo cp ./driver/hvisor.ko /mnt/home/arm64
# 	sudo umount $(DEV)

transfer_nxp: all
	sudo cp ./tools/hvisor ~/tftp
	sudo cp ./tools/ivc_demo ~/tftp
	sudo cp ./driver/hvisor.ko ~/tftp
	sudo cp ./driver/ivc.ko ~/tftp

clean:
	make -C tools clean
	make -C driver clean