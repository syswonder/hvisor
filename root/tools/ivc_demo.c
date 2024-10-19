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
    ivc_info_t ivc_info;
    void *tb_virt, *mem_virt;
    unsigned long long ct_ipa;
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
    ioctl(fd, HVISOR_IVC_INFO, &ivc_info);
    assert(ivc_info.len > 0);
    
    ivc_id = ivc_info.ivc_ids[0];
    ct_ipa = ivc_info.ivc_ct_ipas[0];
    tb_virt = mmap(NULL, 0x1000, PROT_READ | PROT_WRITE, MAP_SHARED, fd, ct_ipa);
    ivc_cttable_t* tb = (ivc_cttable_t* )tb_virt;
    printf("ivc_id: %d, max_peers: %d, shared_mem_ipa: %llx\n", tb->ivc_id, tb->max_peers, tb->shared_mem_ipa);
    mem_virt = mmap(NULL, tb->out_sec_size * tb->max_peers + tb->rw_sec_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, tb->shared_mem_ipa);
    if (is_send) {
        out = mem_virt + tb->peer_id * tb->out_sec_size + tb->rw_sec_size;
        in = mem_virt + tb->rw_sec_size + tb->out_sec_size;
        char *msg = "hello zone1! I'm zone0.";
        strcpy(out, msg);
        tb->ipi_invoke = 1;
        printf("ivc_demo: zone0 sent: %s\n", out);
        sigwait(&wait_set, &sig);
        if (sig == SIGIVC) 
            printf("ivc_demo: zone0 received: %s\n", in);
    } else {
        in = mem_virt + tb->rw_sec_size;
        out = mem_virt + tb->peer_id * tb->out_sec_size + tb->rw_sec_size;
        sigwait(&wait_set, &sig);
        if (sig == SIGIVC) 
            printf("ivc_demo: zone1 received: %s\n", in);
        strcpy(out, "I'm zone1. hello zone0! ");
        tb->ipi_invoke = 0;
        printf("ivc_demo: zone1 sent: %s\n", out);
    }

    close(fd);
    munmap(mem_virt, tb->out_sec_size * tb->max_peers + tb->rw_sec_size);
    munmap(tb_virt, 0x1000);
    return 0;
}