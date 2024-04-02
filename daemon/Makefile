CC = /home/lgw/tools/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu/bin/aarch64-none-linux-gnu-gcc
CFLAGS = -Wall -Wextra -DLOG_USE_COLOR
objects := $(wildcard *.c)
all: 
	$(CC) $(CFLAGS) -o hvisor $(objects) -I../driver/ -I./includes/ -lpthread