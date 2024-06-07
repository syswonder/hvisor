obj-m += hvisor.o

ifeq ($(ARCH), arm64)
	COMPILE := aarch64-none-linux-gnu-
else ifeq ($(ARCH), riscv)
	COMPILE := riscv64-unknown-linux-gnu-
endif

.PHONY: all clean
all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR) M=$(shell pwd) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean