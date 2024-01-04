obj-m += main.o
KDIR = ~/study/hypervisor/linux
COMPILE = ~/tools/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-
ARCH = arm64
# ifdef ARCH
#   #You can update your Beaglebone path here.
#   KDIR = /home/embetronicx/BBG/tmp/lib/modules/5.10.65/build
# else
#   KDIR = /lib/modules/$(shell uname -r)/build
# endif
 
all:
	make ARCH=$(ARCH) CROSS_COMPILE=$(COMPILE) -C $(KDIR)  M=$(shell pwd) modules
 
clean:
	make -C $(KDIR)  M=$(shell pwd) clean