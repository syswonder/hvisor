KDIR ?= ../../nxp/OK8MP-linux-kernel
DEV ?= /dev/sda1
ARCH ?= arm64
LOG ?= LOG_WARN

export KDIR
export ARCH
export LOG
.PHONY: all env tools driver clean

env:
	git submodule update --init --recursive

tools: env
	make -C tools

driver: env
	make -C driver

all: tools driver

transfer: all
	./trans_file.sh ./tools/hvisor 
	./trans_file.sh ./driver/hvisor.ko 

transfer_nxp: all
	# sudo mount $(DEV) /mnt/
	sudo cp ./tools/hvisor ~/tftp
	sudo cp ./driver/hvisor.ko ~/tftp
	# sudo umount $(DEV)

clean:
	make -C tools clean
	make -C driver clean