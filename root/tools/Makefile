CC = aarch64-none-linux-gnu-gcc
CFLAGS = -Wall -Wextra -DLOG_USE_COLOR
objects := $(wildcard *.c)

.PHONY: all clean
all: 
	$(CC) $(CFLAGS) -g -o hvisor $(objects) -I../driver/ -I./includes/ -lpthread

clean:
	rm hvisor