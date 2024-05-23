KDIR ?= ../../linux

export KDIR
.PHONY: all tools driver clean
tools:
	make -C tools

driver:
	make -C driver

all: tools driver

transfer:
	./trans_file.sh ./tools/hvisor 
	./trans_file.sh ./driver/hvisor.ko 

clean:
	-make -C tools clean
	make -C driver clean