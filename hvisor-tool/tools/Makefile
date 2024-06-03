CC = aarch64-linux-gnu-gcc
CFLAGS = -Wall -Wextra -DLOG_USE_COLOR -O2
objects := $(wildcard *.c)

.PHONY: all clean
all: 
	$(CC) $(CFLAGS) -g -o hvisor $(objects) -I../driver/ -I./includes/ -lpthread

clean:
	rm hvisor