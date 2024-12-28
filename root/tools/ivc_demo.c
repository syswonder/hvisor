#include <stdio.h>
#include <sys/ioctl.h>
#include <assert.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <signal.h>
#include <poll.h>
#include <unistd.h>
#include "ivc.h"
volatile char *out, *in;
struct pollfd pfd;

static int open_dev()
{
    int fd = open("/dev/hivc0", O_RDWR);
    if (fd < 0)
    {
        perror("open hvisor failed");
        exit(1);
    }
    return fd;
}

int main(int argc, char *argv[]) {
    printf("ivc_demo: starting\n");
    int fd, err, sig, ivc_id, is_send = 0, ret;
    ivc_uinfo_t ivc_info;
    void *tb_virt, *mem_virt;
    unsigned long long ct_ipa, offset;
    
    if (argc != 2) {
        printf("Usage: ivc_demo send|receive\n");
        return -1;
    }

    if (strcmp(argv[1], "send") == 0) {
        is_send = 1;
    } else if (strcmp(argv[1], "receive") == 0) {
        is_send = 0;
    } else {
        printf("Usage: ivc_demo send|receive\n");
        return -1;
    }
    
    fd = open_dev();

    pfd.fd = fd;
    pfd.events = POLLIN;
    tb_virt = mmap(NULL, 0x1000, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    offset = 0x1000;

    ivc_cttable_t* tb = (ivc_cttable_t* )tb_virt;
    printf("ivc_id: %d, max_peers: %d\n", tb->ivc_id, tb->max_peers);

    if (is_send) {
        out = mmap(NULL, tb->out_sec_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, offset);
        offset += tb->out_sec_size;
        in = mmap(NULL, tb->out_sec_size, PROT_READ, MAP_SHARED, fd, offset);
        char *msg = "hello zone1! I'm zone0.";
        strcpy(out, msg);
        tb->ipi_invoke = 1;
        printf("ivc_demo: zone0 sent: %s\n", out);
        ret = poll(&pfd, 1, -1);
        if (pfd.revents & POLLIN) 
            printf("ivc_demo: zone0 received: %s\n", in);
        else 
            printf("ivc_demo: zone0 poll failed, ret is %d\n", ret);
    } else {
        in = mmap(NULL, tb->out_sec_size, PROT_READ, MAP_SHARED, fd, offset);
        offset += tb->out_sec_size;
        out = mmap(NULL, tb->out_sec_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, offset);
        ret = poll(&pfd, 1, -1);
        if (pfd.revents & POLLIN) 
            printf("ivc_demo: zone1 received: %s\n", in);
        else 
            printf("ivc_demo: zone1 poll failed, ret is %d\n", ret);
        strcpy(out, "I'm zone1. hello zone0! ");
        tb->ipi_invoke = 0;
        printf("ivc_demo: zone1 sent: %s\n", out);
    }

    close(fd);
    munmap(in, tb->out_sec_size);
    munmap(out, tb->rw_sec_size);
    munmap(tb_virt, 0x1000);
    return 0;
}