CC = aarch64-linux-gnu-gcc
CFLAGS = -Wall -Wextra -DLOG_USE_COLOR
objects := $(wildcard *.c)
all: 
	$(CC) $(CFLAGS) -o hvisor $(objects) -I../driver/ -I./includes/ -lpthread

clean:
	rm hvisor