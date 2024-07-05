KDIR ?= ../OK8MP-linux-kernel
DEV ?= /dev/sda1
ARCH ?= arm64

export KDIR
export ARCH

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
	sudo mount $(DEV) /mnt/
	sudo cp ./tools/hvisor /mnt/home/arm64
	sudo cp ./driver/hvisor.ko /mnt/home/arm64
	sudo umount $(DEV)

clean:
	make -C tools clean
	make -C driver clean