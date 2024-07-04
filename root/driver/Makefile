obj-m += hvisor.o
COMPILE = aarch64-none-linux-gnu-
ARCH = arm64

EXTRA_CFLAGS += -I$(PWD)/include

.PHONY: all clean
all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR) M=$(shell pwd) EXTRA_CFLAGS=$(EXTRA_CFLAGS) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean