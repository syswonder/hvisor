CC = aarch64-linux-gnu-gcc
CFLAGS = -Wall -Wextra -DLOG_USE_COLOR
objects = $(wildcard *.c) ../cJSON/cJSON.c

.PHONY: all clean
all: 
	$(CC) $(CFLAGS) -g -o hvisor $(objects) -I./include/ -I../include/ -I../cJSON/ -lpthread

clean:
	rm hvisor