#include <stdio.h>
#include <sys/ioctl.h>
#include <assert.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <signal.h>

#include "ivc.h"
volatile char *out, *in;
static int open_dev()
{
    int fd = open("/dev/hvisorivc", O_RDWR);
    if (fd < 0)
    {
        perror("open hvisor failed");
        exit(1);
    }
    return fd;
}

int main(int argc, char *argv[]) {
    printf("ivc_demo: starting\n");
    int fd, err, sig, ivc_id, is_send = 0;
    ivc_uinfo_t ivc_info;
    void *tb_virt, *mem_virt;
    unsigned long long ct_ipa, offset;
    sigset_t wait_set;
    sigset_t block_mask;
	sigfillset(&block_mask);
	pthread_sigmask(SIG_BLOCK, &block_mask, NULL);
    
    if (strcmp(argv[1], "send") == 0) {
        is_send = 1;
    } else if (strcmp(argv[1], "receive") == 0) {
        is_send = 0;
    } else {
        printf("Usage: ivc_demo send|receive\n");
        return -1;
    }
    
	sigemptyset(&wait_set);
	sigaddset(&wait_set, SIGIVC);
    
    fd = open_dev();
    ioctl(fd, HVISOR_IVC_USER_INFO, &ivc_info);
    assert(ivc_info.len > 0);
    
    ivc_id = ivc_info.ivc_ids[0];
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
        sigwait(&wait_set, &sig);
        if (sig == SIGIVC) 
            printf("ivc_demo: zone0 received: %s\n", in);
    } else {
        in = mmap(NULL, tb->out_sec_size, PROT_READ, MAP_SHARED, fd, offset);
        offset += tb->out_sec_size;
        out = mmap(NULL, tb->out_sec_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, offset);
        sigwait(&wait_set, &sig);
        if (sig == SIGIVC) 
            printf("ivc_demo: zone1 received: %s\n", in);
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