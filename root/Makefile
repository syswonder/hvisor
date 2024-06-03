KDIR ?= ../../linux
# KDIR ?= ~/study/hypervisor/nxp/OK8MP-linux-kernel
DEV ?= /dev/sdb1
export KDIR
.PHONY: all tools driver clean
tools:
	make -C tools

driver:
	make -C driver

all: tools driver

transfer: all
	./trans_file.sh ./tools/hvisor 
	./trans_file.sh ./driver/hvisor.ko 

transfer_nxp: all
	sudo mount $(DEV) /mnt/
	sudo cp ./tools/hvisor /mnt/home/arm64
	sudo cp ./driver/hvisor.ko /mnt/home/arm64
	sudo umount /mnt
	sudo umount $(DEV)
clean:
	-make -C tools clean
	make -C driver clean