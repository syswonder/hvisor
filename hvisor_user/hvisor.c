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

#include "hvisor.h"
#include "virtio.h"
#include "log.h"
int hvisor_init();
void hvisor_sig_handler(int n, siginfo_t *info, void *unused);

struct hvisor_device_region *device_region;
int fd;

int main(int argc, char *argv[])
{
    hvisor_init();
}

int hvisor_init()
{
    int err;
    log_set_level(LOG_DEBUG);
    FILE *log_file = fopen("log.txt", "w+");
    if (log_file == NULL) {
        log_error("open log file failed");
    }
    log_add_fp(log_file, 0);
    log_info("hvisor init");
    fd = open("/dev/hvisor", O_RDWR);
    if (fd < 0) {
        log_error("open hvisor failed");
        exit(1);
    }
    // ioctl for init virtio
    err = ioctl(fd, HVISOR_INIT_VIRTIO);
    if (err) {
        log_error("ioctl failed, err code is %d", err);
        close(fd);
        exit(1);
    }

    // mmap: create shared memory
    device_region = (struct hvisor_device_region *) mmap(NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (device_region == (void *)-1) {
        log_error("mmap failed");
        goto unmap;
    }

    // register signal handler
    struct sigaction act;
    sigset_t block_mask;
    sigfillset(&block_mask);
    act.sa_flags = SA_SIGINFO;
    act.sa_sigaction = hvisor_sig_handler;
    // If one signal A is being handled, another signal B occurs, signal B will be blocked until signal A is finished.
    // If there are five signal B, only handle once. 
    act.sa_mask = block_mask;
    if (sigaction(SIGHVI, &act, NULL) == -1) 
        log_error("register signal handler failed");

    init_virtio_devices();
    log_info("hvisor init okay!");
    while(1);

unmap:
    munmap(device_region, MMAP_SIZE);
    return 0;
}

void hvisor_sig_handler(int n, siginfo_t *info, void *unused)
{
    log_trace("received one signal %d", n);
    if (n == SIGHVI) {
        unsigned int nreq = device_region->nreq;
        log_trace("nreq is %u", nreq);
        while (nreq--) {
            struct device_req *req = &device_region->req_list[nreq];
            struct device_result *res = &device_region->res;
            virtio_handle_req(req, res);
            device_region->nreq --;
            ioctl(fd, HVISOR_FINISH);
        }
    }
}