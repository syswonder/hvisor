obj-m += hvisor.o
obj-m += ivc.o
ccflags-y += -I$(src)/../include
ccflags-y += -Wno-unused-variable
ccflags-y += -Wno-unused-function
ccflags-y += -fno-stack-protector

ifndef CROSS_COMPILE
	ifeq ($(ARCH), arm64)
		CROSS_COMPILE := aarch64-none-linux-gnu-
	else ifeq ($(ARCH), riscv)
		CROSS_COMPILE := riscv64-unknown-linux-gnu-
	else ifeq ($(ARCH), loongarch)
		CROSS_COMPILE := loongarch64-unknown-linux-gnu-
	else ifeq ($(ARCH), x86_64)
		CROSS_COMPILE := x86_64-linux-gnu-
	endif
endif

ifeq ($(ARCH), loongarch)
	ccflags-y += -DLOONGARCH64
endif

ivc-y := ivc_driver.o

.SECONDARY: \
        $(obj)/hivc_template.dtb.S \
        $(obj)/hivc_template.dtb

.PHONY: all clean

all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) -C $(KDIR) M=$(shell pwd) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean
