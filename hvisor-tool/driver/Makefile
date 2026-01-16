obj-m += hvisor.o
obj-m += ivc.o
ccflags-y += -I$(src)/../include
ccflags-y += -Wno-unused-variable
ccflags-y += -Wno-unused-function
ccflags-y += -fno-stack-protector

ifeq ($(ARCH), arm64)
	COMPILE := aarch64-none-linux-gnu-
	OBJDUMP := aarch64-none-linux-gnu-objdump
else ifeq ($(ARCH), riscv)
	COMPILE := riscv64-linux-gnu-
	OBJDUMP := riscv64-linux-gnu-objdump
else ifeq ($(ARCH), loongarch)
	COMPILE := loongarch64-unknown-linux-gnu-
	OBJDUMP := loongarch64-unknown-linux-gnu-objdump
	EXTRA_CFLAGS += -DLOONGARCH64
else ifeq ($(ARCH), x86_64)
	COMPILE := x86_64-linux-gnu-
	OBJDUMP := x86_64-linux-gnu-objdump
else
	COMPILE := 
endif

ivc-y := ivc_driver.o

.SECONDARY: \
        $(obj)/hivc_template.dtb.S \
        $(obj)/hivc_template.dtb

.PHONY: all clean

all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR) M=$(shell pwd) modules
 
clean:
	make -C $(KDIR) M=$(shell pwd) clean