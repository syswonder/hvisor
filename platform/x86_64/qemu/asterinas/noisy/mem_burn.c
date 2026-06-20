/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include "../probes/common.h"

#include <signal.h>
#include <sys/mman.h>

static volatile sig_atomic_t stop;

static void on_signal(int signo)
{
    (void)signo;
    stop = 1;
}

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--chunk-mb N] [--seconds N]\n"
            "\n"
            "Repeatedly mmap/memset/munmap memory to stress TLB and DRAM bandwidth.\n",
            argv0);
}

int main(int argc, char **argv)
{
    long chunk_mb = 64;
    long seconds = 300;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--chunk-mb") == 0 && i + 1 < argc) {
            chunk_mb = parse_long_arg("--chunk-mb", argv[++i], 1, 4096);
        } else if (strcmp(argv[i], "--seconds") == 0 && i + 1 < argc) {
            seconds = parse_long_arg("--seconds", argv[++i], 1, 86400);
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            usage(argv[0]);
            return 0;
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    signal(SIGINT, on_signal);
    signal(SIGTERM, on_signal);

    size_t len = (size_t)chunk_mb * 1024u * 1024u;
    uint64_t deadline = nsec_now_monotonic() + (uint64_t)seconds * 1000000000ull;
    unsigned long loops = 0;
    while (!stop && nsec_now_monotonic() < deadline) {
        void *ptr = mmap(NULL, len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (ptr == MAP_FAILED) {
            die_errno("mmap");
        }
        memset(ptr, (int)(loops & 0xffu), len);
        if (munmap(ptr, len) != 0) {
            die_errno("munmap");
        }
        loops++;
    }

    printf("mem_burn loops=%lu chunk_mb=%ld seconds=%ld\n", loops, chunk_mb, seconds);
    return 0;
}
