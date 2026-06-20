/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

/*
 * noisy_init: PID 1 of the noisy-neighbour zone (zone2). It saturates the
 * pCPUs assigned to this zone and hammers the shared micro-architectural
 * resources (LLC + DRAM bandwidth) that a statically-partitioned hypervisor
 * cannot fence off, so zone1 probe runs can check IPI and timer behaviour under
 * interference.
 *
 * Load generated per online CPU:
 *   - a tight ALU loop (compute pressure);
 *   - a streaming write over a multi-megabyte buffer (LLC + DRAM bandwidth).
 * Plus one /bin/mem_burn child for sustained mmap/memset/munmap churn (TLB).
 *
 * It is intentionally self-contained (no shell / busybox) so it runs as a
 * minimal initramfs /init on either Linux or Asterinas (Linux ABI).
 */

#define _GNU_SOURCE

#include <sched.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include <fcntl.h>
#include <sys/mount.h>
#include <sys/stat.h>

#define THRASH_BYTES (8u * 1024u * 1024u) /* ~8 MiB to spill the LLC */

static void attach_console(void)
{
    int fd = open("/dev/console", O_RDWR);
    if (fd < 0) {
        fd = open("/dev/ttyS0", O_RDWR);
    }
    if (fd >= 0) {
        dup2(fd, 0);
        dup2(fd, 1);
        dup2(fd, 2);
        if (fd > 2) {
            close(fd);
        }
    }
}

static void bind_cpu(int cpu)
{
    cpu_set_t set;
    CPU_ZERO(&set);
    CPU_SET(cpu, &set);
    (void)sched_setaffinity(0, sizeof(set), &set);
}

/* Never returns: pure interference generator pinned to one CPU. */
static void burn_forever(int cpu)
{
    bind_cpu(cpu);
    volatile uint64_t acc = 0x9e3779b97f4a7c15ull ^ (uint64_t)cpu;
    unsigned char *buf = malloc(THRASH_BYTES);
    size_t step = 64; /* one cache line */
    for (;;) {
        /* ALU pressure. */
        for (int i = 0; i < 4096; i++) {
            acc = acc * 6364136223846793005ull + 1442695040888963407ull;
        }
        /* Streaming write: LLC + DRAM bandwidth contention. */
        if (buf) {
            for (size_t off = 0; off < THRASH_BYTES; off += step) {
                buf[off] = (unsigned char)(acc >> (off & 7));
            }
        }
    }
}

int main(void)
{
    attach_console();
    (void)mkdir("/proc", 0755);
    (void)mount("proc", "/proc", "proc", 0, NULL);

    long ncpu = sysconf(_SC_NPROCESSORS_ONLN);
    if (ncpu < 1) {
        ncpu = 1;
    }

    printf("noisy zone init: generating interference on %ld cpu(s)\n", ncpu);
    fflush(stdout);

    for (long cpu = 0; cpu < ncpu; cpu++) {
        pid_t pid = fork();
        if (pid == 0) {
            burn_forever((int)cpu);
            _exit(0);
        }
    }

    /* Memory-churn child (TLB + allocator pressure), if present. */
    pid_t mem = fork();
    if (mem == 0) {
        char *argv[] = {"/bin/mem_burn", "--chunk-mb", "128",
                        "--seconds", "86400", NULL};
        execv(argv[0], argv);
        /* mem_burn missing: fall back to extra ALU/bandwidth burn. */
        burn_forever(0);
        _exit(0);
    }

    printf("noisy zone init: workers launched, entering idle reap loop\n");
    fflush(stdout);
    for (;;) {
        int status = 0;
        pid_t done = wait(&status);
        if (done < 0) {
            struct timespec ts = {.tv_sec = 3600, .tv_nsec = 0};
            nanosleep(&ts, NULL);
        }
    }
    return 0;
}
