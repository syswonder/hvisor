obj-m += hvisor.o
obj-m += ivc.o
EXTRA_CFLAGS += "

ifeq ($(ARCH), arm64)
	COMPILE := aarch64-none-linux-gnu-
	OBJDUMP := aarch64-none-linux-gnu-objdump
else ifeq ($(ARCH), riscv)
	COMPILE := riscv64-unknown-linux-gnu-
	OBJDUMP := riscv64-unknown-linux-gnu-objdump
else ifeq ($(ARCH), loongarch)
	COMPILE := loongarch64-unknown-linux-gnu-
	OBJDUMP := loongarch64-unknown-linux-gnu-objdump
	EXTRA_CFLAGS += -DLOONGARCH64
else
	COMPILE := 
endif

EXTRA_CFLAGS += -I$(PWD)/include -Werror -Wall

ivc-y := ivc_driver.o

.SECONDARY: \
        $(obj)/hivc_template.dtb.S \
        $(obj)/hivc_template.dtb

EXTRA_CFLAGS += "

.PHONY: all clean

all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR) M=$(shell pwd) EXTRA_CFLAGS=$(EXTRA_CFLAGS) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean