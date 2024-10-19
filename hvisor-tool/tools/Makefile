CFLAGS = -Wall -Wextra -DLOG_USE_COLOR -DHLOG=$(LOG)
objects = $(filter-out ivc_demo.c, $(wildcard *.c) ../cJSON/cJSON.c)

ifeq ($(ARCH), arm64)
	CC := aarch64-linux-gnu-gcc
else ifeq ($(ARCH), riscv)
	CC := riscv64-linux-gnu-gcc
endif

.PHONY: all clean
all: hvisor ivc

hvisor: 
	$(CC) $(CFLAGS) -g -o hvisor $(objects) -I./include/ -I../include/ -I../cJSON/ -lpthread

ivc:
	$(CC) -Wall -Wextra -g -o ivc_demo ivc_demo.c  -I../include/

asm:
	$(CC) $(CFLAGS) -S htool.s $(objects) -I../driver/ -I./includes/ -lpthread
clean:
	rm -f hvisor