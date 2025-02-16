sources := $(wildcard *.c) ../cJSON/cJSON.c
objects := $(sources:.c=.o)
ivc_demo_object := ivc_demo.o
rpmsg_demo_object := rpmsg_demo.o
hvisor_objects := $(filter-out $(ivc_demo_object) $(rpmsg_demo_object), $(objects))

CFLAGS := -Wall -Wextra -DLOG_USE_COLOR -DHLOG=$(LOG)
include_dirs := -I../include -I./include -I../cJSON/ -lpthread

include $(sources:.c=.d)

ifeq ($(DEBUG), y)
	CFLAGS += -g -O0
else 
	CFLAGS += -O2
endif

ifeq ($(ARCH), arm64)
	CC := aarch64-linux-gnu-gcc
else ifeq ($(ARCH), riscv)
	CC := riscv64-linux-gnu-gcc
endif

.PHONY: all clean

all: hvisor ivc_demo rpmsg_demo

%.d: %.c
	@set -e; rm -f $@; \
	$(CC) -MM $(include_dirs) $< > $@.$$$$; \
	sed 's,\($*\)\.o[ :]*,\1.o $@ : ,g' < $@.$$$$ > $@; \
	rm -f $@.$$$$

$(objects): %.o: %.c
	$(CC) $(CFLAGS) $(include_dirs) -c -o $@ $<

hvisor: $(hvisor_objects)
	$(CC) -o $@ $^ $(include_dirs)

ivc_demo: $(ivc_demo_object)
	$(CC) -o $@ $^ $(include_dirs)

rpmsg_demo: $(rpmsg_demo_object)
	$(CC) -o $@ $^ $(include_dirs)

clean:
	rm -f hvisor ivc_demo *.o *.d *.d.* 