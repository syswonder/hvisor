ROOT_DIR := ..
COMMON_DIR := $(ROOT_DIR)/common
TOOL_DIR := tool
DAEMON_DIR := daemon/virtio
CJSON_DIR := $(COMMON_DIR)/third_party/cJSON

sources := $(wildcard $(COMMON_DIR)/*.c) \
           $(wildcard $(TOOL_DIR)/*.c) \
           $(wildcard $(TOOL_DIR)/boot/*.c) \
           $(wildcard $(DAEMON_DIR)/*.c) \
           $(wildcard $(DAEMON_DIR)/devices/blk/*.c) \
           $(wildcard $(DAEMON_DIR)/devices/console/*.c) \
           $(wildcard $(DAEMON_DIR)/devices/net/*.c) \
           $(wildcard $(DAEMON_DIR)/devices/scmi/*.c) \
           $(CJSON_DIR)/cJSON.c

ifeq ($(VIRTIO_GPU), y)
sources += $(wildcard $(DAEMON_DIR)/devices/gpu/*.c)
endif

objects := $(sources:.c=.o)
ivc_demo_object := $(TOOL_DIR)/ivc_demo.o
rpmsg_demo_object := $(TOOL_DIR)/rpmsg_demo.o
hvisor_objects := $(filter-out $(ivc_demo_object) $(rpmsg_demo_object), $(objects))
BUILD_DEMOS ?= n
ROOT ?=
LIBC ?= gnu

ifeq ($(filter $(LIBC),gnu musl),)
$(error LIBC must be one of: gnu musl)
endif

ifndef CROSS_COMPILE
	ifeq ($(ARCH), arm64)
		CROSS_COMPILE := aarch64-none-linux-$(LIBC)-
	else ifeq ($(ARCH), riscv)
		CROSS_COMPILE := riscv64-linux-$(LIBC)-
	else ifeq ($(ARCH), loongarch)
		CROSS_COMPILE := loongarch64-unknown-linux-$(LIBC)-
	else ifeq ($(ARCH), x86_64)
		CROSS_COMPILE := x86_64-linux-$(LIBC)-
	else
$(error "Unsupported architecture $(ARCH)")
	endif
endif

CFLAGS := -Wall -Wextra -DHLOG=$(LOG) -pthread -MMD -MP
ifneq ($(ROOT),)
CFLAGS += --sysroot=$(ROOT)
endif

include_dirs := -I$(COMMON_DIR)/include -I$(COMMON_DIR) \
                -I$(TOOL_DIR)/include -I$(TOOL_DIR)/boot \
                -I$(DAEMON_DIR)/include -I$(CJSON_DIR) -I$(TOOL_DIR)

ifeq ($(LIBC), musl)
include_dirs += -I$(TOOL_DIR)/compat/
endif

LDFLAGS := -lpthread -static

ifeq ($(VIRTIO_GPU), y)
	CFLAGS += -DENABLE_VIRTIO_GPU
endif
ifeq ($(VIRTIO_SCMI), y)
	CFLAGS += -DENABLE_VIRTIO_SCMI
endif
ifeq ($(DEBUG), y)
	CFLAGS += -g -O0
else
	CFLAGS += -O2
endif

ifeq ($(origin CC), default)
CC := $(CROSS_COMPILE)gcc
endif

ifeq ($(ARCH), arm64)
	ifeq ($(VIRTIO_GPU), y)
		triplet := $(patsubst %-,%,$(notdir $(CROSS_COMPILE)))
		include_dirs += -I/usr/$(triplet)/include -I/usr/$(triplet)/include/libdrm -L/usr/$(triplet)/lib -ldrm
	endif
else ifeq ($(ARCH), riscv)
	CFLAGS += -static
else ifeq ($(ARCH), loongarch)
	CFLAGS += -DLOONGARCH64 -static
	ifeq ($(VIRTIO_GPU), y)
		include_dirs += -I/opt/libdrm-install/include -L/opt/libdrm-install/lib -I/opt/libdrm-install/include/libdrm -ldrm
	endif
endif

ifeq ($(BUILD_DEMOS),y)
demo_targets := $(TOOL_DIR)/ivc_demo $(TOOL_DIR)/rpmsg_demo
endif

.PHONY: all tools driver clean check-kdir

all: tools driver

tools: $(TOOL_DIR)/hvisor $(demo_targets)

$(TOOL_DIR)/hvisor: $(hvisor_objects)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(TOOL_DIR)/ivc_demo: $(ivc_demo_object)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(TOOL_DIR)/rpmsg_demo: $(rpmsg_demo_object)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(objects): %.o: %.c
	$(CC) $(CFLAGS) $(include_dirs) $(LIBS) -c -o $@ $<

check-kdir:
ifeq ($(KDIR),)
	$(error Linux kernel directory is not set. Please set environment variable 'KDIR')
endif

driver: check-kdir
	$(MAKE) -C driver all VIRTIO_SCMI=$(VIRTIO_SCMI)

clean:
	@rm -f $(objects) $(objects:.o=.d) \
		$(TOOL_DIR)/hvisor $(TOOL_DIR)/ivc_demo $(TOOL_DIR)/rpmsg_demo
	@$(MAKE) -C driver clean KDIR=$(KDIR) 2>/dev/null || true

-include $(objects:.o=.d)
