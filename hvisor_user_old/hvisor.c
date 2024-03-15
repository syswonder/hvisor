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
#include "tools.h"
int hvisor_init();
// void hvisor_sig_handler(int n, siginfo_t *info, void *unused);
void handle_virtio_requests();

/// hvisor kernel module fd
int ko_fd;
volatile struct hvisor_device_region *device_region;

int main()
{
    hvisor_init();
}

int hvisor_init()
{
    // The higher log level is , faster virtio-blk will be.
    int err;
    log_set_level(LOG_DEBUG);
    FILE *log_file = fopen("log.txt", "w+");
    if (log_file == NULL) {
        log_error("open log file failed");
    }
    log_add_fp(log_file, LOG_DEBUG);
    log_info("hvisor init");
    ko_fd = open("/dev/hvisor", O_RDWR);
    if (ko_fd < 0) {
        log_error("open hvisor failed");
        exit(1);
    }
    // ioctl for init virtio
    err = ioctl(ko_fd, HVISOR_INIT_VIRTIO);
    if (err) {
        log_error("ioctl failed, err code is %d", err);
        close(ko_fd);
        exit(1);
    }

    // mmap: create shared memory
    device_region = (struct hvisor_device_region *) mmap(NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, ko_fd, 0);
    if (device_region == (void *)-1) {
        log_error("mmap failed");
        goto unmap;
    }

    mevent_init();
    init_virtio_devices();
    log_info("hvisor init okay!");
    handle_virtio_requests();

unmap:
    munmap((void *)device_region, MMAP_SIZE);
    return 0;
}

void handle_virtio_requests()
{
    unsigned int req_front = device_region->req_front;
    volatile struct device_req *req;
//    int flag = 0;
    while (1) {
        if (!is_queue_empty(req_front, device_region->req_rear)) {
            req = &device_region->req_list[req_front];
            virtio_handle_req(req);
            req_front = (req_front + 1) & (MAX_REQ - 1);
            device_region->req_front = req_front;
            // dmb_ishst(); 应该不需要
        }
    }
}