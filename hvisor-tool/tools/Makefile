sources ?= $(wildcard *.c) ../cJSON/cJSON.c
objects ?= $(sources:.c=.o)
ivc_demo_object ?= ivc_demo.o
rpmsg_demo_object ?= rpmsg_demo.o
hvisor_objects ?= $(filter-out $(ivc_demo_object) $(rpmsg_demo_object), $(objects))
ROOT ?= /
# gnu or musl
LIBC ?= gnu

ifeq ($(ARCH), arm64)
	CCARCH := aarch64
else ifeq ($(ARCH), riscv)
	CCARCH := riscv64
else ifeq ($(ARCH), loongarch)
	CCARCH := loongarch64
else ifeq ($(ARCH), x86_64)
	CCARCH := x86_64
else
$(error "Unsupported architecture $(ARCH)")
endif

ifeq ($(filter $(LIBC),gnu musl),)
$(error LIBC must be one of: gnu musl)
endif

ifeq ($(ARCH), loongarch)
toolchain := $(CCARCH)-unknown-linux-$(LIBC)
else ifeq ($(ARCH), arm64)
toolchain := $(CCARCH)-none-linux-$(LIBC)
else
toolchain := $(CCARCH)-linux-$(LIBC)
endif

CFLAGS := -Wall -Wextra -DLOG_USE_COLOR -DHLOG=$(LOG) --sysroot=$(ROOT)

include_dirs := -I../include -I./include -I../cJSON

ifeq ($(LIBC), musl)
include_dirs += -I./compat/
endif

LDFLAGS := -lpthread

# LIBS := -L$(ROOT)/usr/lib/$(toolchain) -L$(ROOT)/lib
# LIBS := -L/opt/aarch64-linux-musl-cross/aarch64-linux-musl -L/opt/aarch64-linux-musl-cross/lib

ifeq ($(VIRTIO_GPU), y)
	sources += $(wildcard ./virtio_gpu/*.c)
	CFLAGS += -DENABLE_VIRTIO_GPU
endif

include $(sources:.c=.d)

ifeq ($(DEBUG), y)
	CFLAGS += -g -O0
else
	CFLAGS += -O2
endif

CC := $(toolchain)-gcc
READELF := $(toolchain)-readelf
OBJDUMP := $(toolchain)-objdump

ifeq ($(ARCH), arm64)
	ifeq ($(VIRTIO_GPU), y)
		include_dirs += -I/usr/$(toolchain)/include -I/usr/$(toolchain)/include/libdrm -L/usr/$(toolchain)/lib -ldrm
	endif
else ifeq ($(ARCH), riscv)
	CFLAGS += -static
else ifeq ($(ARCH), loongarch)
	CFLAGS += -DLOONGARCH64 -static
	ifeq ($(VIRTIO_GPU), y)
		include_dirs += -I/opt/libdrm-install/include -L/opt/libdrm-install/lib -I/opt/libdrm-install/include/libdrm -ldrm
	endif
else ifeq ($(ARCH), x86_64)
	CC := x86_64-linux-gnu-gcc
	READELF := x86_64-linux-gnu-readelf
	OBJDUMP := x86_64-linux-gnu-objdump
endif

.PHONY: all clean

all: hvisor ivc_demo rpmsg_demo

%.d: %.c
	@set -e; rm -f $@; \
	$(CC) -MM $(include_dirs) $< > $@.$$$$; \
	sed 's,\($*\)\.o[ :]*,\1.o $@ : ,g' < $@.$$$$ > $@; \
	rm -f $@.$$$$

$(objects): %.o: %.c
	$(CC) $(CFLAGS) $(include_dirs) $(LIBS) -c -o $@ $<

hvisor: $(hvisor_objects)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

ivc_demo: $(ivc_demo_object)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

rpmsg_demo: $(rpmsg_demo_object)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

clean:
	@rm -f hvisor ivc_demo rpmsg_demo *.o *.d *.d.* virtio_gpu/*.o virtio_gpu/*.d virtio_gpu/*.d.* ../cJSON/*.o ../cJSON/*.d ../cJSON/*.d.*
