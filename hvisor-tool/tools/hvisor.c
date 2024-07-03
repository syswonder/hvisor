#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <signal.h>
#include <pthread.h>
#include "hvisor.h"
#include "virtio.h"
#include "log.h"
#include "event_monitor.h"
#include <errno.h>
#include <getopt.h>

static void __attribute__((noreturn)) help(int exit_status) {
    printf("Invalid Parameters!\n");
    exit(exit_status);
}

static void* read_file(char* filename, unsigned long long* filesize){
    int fd;
    struct stat st;
    void *buf;
    ssize_t len;
    fd = open(filename, O_RDONLY);

    if(fd < 0) {
        perror("read_file: open file failed");
        exit(1);
    }

    if (fstat(fd, &st) < 0) {
        perror("read_file: fstat failed");
        exit(1);
    }
	long page_size = sysconf(_SC_PAGESIZE);
	ssize_t buf_size = (st.st_size + page_size - 1) & ~(page_size - 1);
    buf = malloc(buf_size);
    memset(buf, 0, buf_size);
	len = read(fd, buf, st.st_size);
	printf("len is %ld, st_size is %ld\n", len, st.st_size);

    if (len < 0) {
        perror("read_file: read failed");
        exit(1);
    }
    if (filesize)
        *filesize = len;
    close(fd);
    return buf;
}

int open_dev() {
    int fd = open("/dev/hvisor", O_RDWR);
    if (fd < 0) {
        perror("open hvisor failed");
        exit(1);
    }
    return fd;
}

static void get_info(char *optarg, char **path, unsigned long long *address) {
	char *now;
	*path = strtok(optarg, ",");
	now = strtok(NULL, "=");
	if (strcmp(now, "addr") == 0) {
		now = strtok(NULL, "=");
		*address = strtoull(now, NULL, 16);
	} else {
		help(1);
	}
}
// ./hvisor zone start -kernel image.bin 0x1000 -dtb image.dtb 0x2000 -id 1
static int zone_start(int argc, char *argv[]) {
	static struct option long_options[] = {
		{"kernel", required_argument, 0, 'k'},
		{"dtb", required_argument, 0, 'd'},
		{"id", required_argument, 0, 'i'},
		{0, 0, 0, 0}
	};
	char *optstring = "k:d:i:";

	struct hvisor_zone_info *zone_info;
    int fd, err, opt, zone_id;
	char *image_path = NULL, *dtb_path = NULL;
	unsigned long long image_address, dtb_address;
	unsigned long long virt_addrs[2], phys_addrs[2], image_sizes[2];
	zone_id = 0;
	image_address = dtb_address = 0;

	while ((opt = getopt_long(argc, argv, optstring, long_options, NULL)) != -1) {
		switch (opt) {
			case 'k':
				get_info(optarg, &image_path, &image_address);
				break;
			case 'd':
				get_info(optarg, &dtb_path, &dtb_address);
				break;
			case 'i':
				sscanf(optarg, "%d", &zone_id);
				break;
			default:
				help(1);
		}
	}
	if (image_path == NULL || dtb_path == NULL || zone_id == 0 || image_address == 0 || dtb_address == 0) {
		help(1);
	}
	zone_info = malloc(sizeof(struct hvisor_zone_info));
	zone_info->zone_id = zone_id;
	zone_info->image_phys_addr = phys_addrs[0] = image_address;
	zone_info->dtb_phys_addr = phys_addrs[1] = dtb_address;

    virt_addrs[0] = (unsigned long long) read_file(image_path, &image_sizes[0]);
    virt_addrs[1] = (unsigned long long) read_file(dtb_path, &image_sizes[1]);
	
	zone_info->image_size = image_sizes[0];
	zone_info->dtb_size = image_sizes[1];
	long page_size = sysconf(_SC_PAGESIZE);
    if (page_size == -1) {
        perror("sysconf");
        exit(EXIT_FAILURE);
    }
	int mem_fd = open("/dev/hvisor", O_RDWR);
	if(mem_fd < 0) {
		printf("open hvisor failed\n");
		exit(1);
	}
	for(int i = 0; i < 2; i++) {
		size_t map_size = (image_sizes[i] + page_size - 1) & ~(page_size - 1);
		void *virt_addr = mmap(NULL, map_size, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_SHARED, mem_fd, phys_addrs[i]);
		memcpy(virt_addr, (void *)virt_addrs[i], map_size);
		if(munmap(virt_addr, map_size) == -1) {
			printf("munmap failed\n");
		}
        free((void*) virt_addrs[i]);
	}
	close(mem_fd);

	printf("success to map kernel and dtb to target address\n");
    fd = open_dev();
    err = ioctl(fd, HVISOR_ZONE_START, zone_info);
    if (err)
        perror("zone_start: ioctl failed");
    close(fd);
    return err;
}

// ./hvisor zone shutdown -id 1
static int zone_shutdown(int argc, char *argv[]) {
	if (argc != 2 || strcmp(argv[0], "-id") != 0) {
        help(1);
	}
	__u64 zone_id;
	sscanf(argv[1], "%llu", &zone_id);
	int fd = open_dev();
	int err = ioctl(fd, HVISOR_ZONE_SHUTDOWN, zone_id);
	if (err)
		perror("zone_shutdown: ioctl failed");
	close(fd);
	return err;
}

int main(int argc, char *argv[])
{
    int err;

    if (argc < 2)
        help(1);

    if (strcmp(argv[1], "zone") == 0 && strcmp(argv[2], "start") == 0) {
        err = zone_start(argc, argv);
    } else if (strcmp(argv[1], "zone") == 0 && strcmp(argv[2], "shutdown") == 0){
		err = zone_shutdown(argc - 3, &argv[3]);
	}else if (strcmp(argv[1], "virtio") == 0 && strcmp(argv[2], "start") == 0) {
        err = virtio_start(argc, argv);
    } else {
        help(1);
    }

    return err ? 1 : 0;
}



