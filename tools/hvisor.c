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
#include "mevent.h"
#include <errno.h>

static void __attribute__((noreturn)) help(int exit_status) {
    printf("Invalid Parameters!\n");
    exit(exit_status);
}

static void* read_file(char* filename, int* filesize){
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

    buf = malloc(st.st_size);
    len = read(fd, buf, st.st_size);
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

// ./hvisor zone start -kernel image.bin 0x1000 -dtb image.dtb 0x2000 -id 1
static int zone_start(int argc, char *argv[]) {
    struct hvisor_zone_load *zone_load;
    struct hvisor_image_desc *images;
    int fd, err;
    if (argc < 8) {
        help(1);
    }
    if (strcmp(argv[0], "-kernel") != 0 || strcmp(argv[3], "-dtb") != 0 || strcmp(argv[6], "-id") != 0)  {
        help(1);
    }
    zone_load = malloc(sizeof(struct hvisor_zone_load));
    zone_load->images_num = 2;
    images = malloc(sizeof(struct hvisor_image_desc)*2);

    images[0].source_address = read_file(argv[1], &images[0].size);
    images[1].source_address = read_file(argv[4], &images[1].size);
    sscanf(argv[2], "%llx", &images[0].target_address);
    sscanf(argv[5], "%llx", &images[1].target_address);
	sscanf(argv[7], "%llu", &zone_load->zone_id);
    zone_load->images = images;
    fd = open_dev();
    err = ioctl(fd, HVISOR_ZONE_START, zone_load);
    if (err)
        perror("zone_start: ioctl failed");
    close(fd);
    for (int i = 0; i < zone_load->images_num; i++)
        free(images[i].source_address);
    free(images);
    free(zone_load);
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
        err = zone_start(argc - 3, &argv[3]);
    } else if (strcmp(argv[1], "zone") == 0 && strcmp(argv[2], "shutdown") == 0){
		err = zone_shutdown(argc - 3, &argv[3]);
	}else if (strcmp(argv[1], "virtio") == 0) {
        err = virtio_init();
    } else {
        help(1);
    }

    return err ? 1 : 0;
}



