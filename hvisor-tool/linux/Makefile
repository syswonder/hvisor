ROOT_DIR := ..
COMMON_DIR := $(ROOT_DIR)/common
ABI_DIR := $(ROOT_DIR)/../abi/include
TOOL_DIR := tool
DAEMON_DIR := daemon
VIRTIO_DIR := $(DAEMON_DIR)/virtio
CJSON_DIR := $(COMMON_DIR)/third_party/cJSON

COMMON_SOURCES := $(wildcard $(COMMON_DIR)/*.c)
TOOL_SOURCES := $(wildcard $(TOOL_DIR)/*.c) \
                $(wildcard $(TOOL_DIR)/boot/*.c)
DAEMON_SOURCES := $(wildcard $(VIRTIO_DIR)/*.c) \
                  $(wildcard $(VIRTIO_DIR)/devices/blk/*.c) \
                  $(wildcard $(VIRTIO_DIR)/devices/console/*.c) \
                  $(wildcard $(VIRTIO_DIR)/devices/net/*.c) \
                  $(wildcard $(VIRTIO_DIR)/devices/scmi/*.c) \
                  $(DAEMON_DIR)/main.c

ifeq ($(VIRTIO_GPU), y)
DAEMON_SOURCES += $(wildcard $(VIRTIO_DIR)/devices/gpu/*.c)
endif

COMMON_OBJECTS := $(COMMON_SOURCES:.c=.o)
TOOL_OBJECTS := $(TOOL_SOURCES:.c=.o)
DAEMON_OBJECTS := $(DAEMON_SOURCES:.c=.o)
CJSON_OBJECT := $(CJSON_DIR)/cJSON.o
TOOL_HVISOR_OBJECTS := $(filter-out $(TOOL_DIR)/ivc_demo.o $(TOOL_DIR)/rpmsg_demo.o,$(TOOL_OBJECTS))

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

include_dirs := -I$(COMMON_DIR)/include -I$(COMMON_DIR) -I$(ABI_DIR) \
                -I$(TOOL_DIR)/include -I$(TOOL_DIR)/boot \
                -I$(VIRTIO_DIR)/include -I$(CJSON_DIR) -I$(TOOL_DIR)

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

.PHONY: all tools daemon driver clean check-kdir

all: tools daemon

tools: $(TOOL_DIR)/hvisor $(demo_targets)

daemon: $(DAEMON_DIR)/hvisor-virtio-daemon

$(TOOL_DIR)/hvisor: $(COMMON_OBJECTS) $(TOOL_HVISOR_OBJECTS) $(CJSON_OBJECT)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(DAEMON_DIR)/hvisor-virtio-daemon: $(COMMON_OBJECTS) $(DAEMON_OBJECTS) $(CJSON_OBJECT)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(TOOL_DIR)/ivc_demo: $(COMMON_OBJECTS) $(TOOL_DIR)/ivc_demo.o $(CJSON_OBJECT)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(TOOL_DIR)/rpmsg_demo: $(COMMON_OBJECTS) $(TOOL_DIR)/rpmsg_demo.o $(CJSON_OBJECT)
	$(CC) -o $@ $^ $(CFLAGS) $(include_dirs) $(LDFLAGS) $(LIBS)

$(COMMON_OBJECTS) $(TOOL_OBJECTS) $(DAEMON_OBJECTS) $(CJSON_OBJECT): %.o: %.c
	$(CC) $(CFLAGS) $(include_dirs) $(LIBS) -c -o $@ $<

check-kdir:
ifeq ($(KDIR),)
	$(error Linux kernel directory is not set. Please set environment variable 'KDIR')
endif

driver: check-kdir
	$(MAKE) -C driver all VIRTIO_SCMI=$(VIRTIO_SCMI)

clean:
	@rm -f $(COMMON_OBJECTS) $(COMMON_OBJECTS:.o=.d) \
		$(TOOL_OBJECTS) $(TOOL_OBJECTS:.o=.d) \
		$(DAEMON_OBJECTS) $(DAEMON_OBJECTS:.o=.d) \
		$(CJSON_OBJECT) $(CJSON_OBJECT:.o=.d) \
		$(TOOL_DIR)/hvisor $(TOOL_DIR)/ivc_demo $(TOOL_DIR)/rpmsg_demo \
		$(DAEMON_DIR)/hvisor-virtio-daemon
	@$(MAKE) -C driver clean KDIR=$(KDIR) 2>/dev/null || true

-include $(COMMON_OBJECTS:.o=.d) $(TOOL_OBJECTS:.o=.d) $(DAEMON_OBJECTS:.o=.d) $(CJSON_OBJECT:.o=.d)
