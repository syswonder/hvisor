obj-m += hvisor.o
COMPILE = aarch64-none-linux-gnu-
ARCH = arm64

.PHONY: all clean
all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR) M=$(shell pwd) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean