/* SPDX-License-Identifier: MulanPSL-2.0 */
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static uint64_t now_ns(void)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        perror("clock_gettime");
        exit(2);
    }
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

static int cmp_u64(const void *a, const void *b)
{
    uint64_t av = *(const uint64_t *)a;
    uint64_t bv = *(const uint64_t *)b;
    return (av > bv) - (av < bv);
}

static void do_fixed_work(void)
{
    volatile uint64_t acc = 0;
    for (uint64_t i = 0; i < 4096; i++) {
        acc += (i * 2654435761ULL) ^ (acc >> 3);
    }
}

int main(int argc, char **argv)
{
    size_t samples = 10000;
    if (argc >= 2) {
        char *end = NULL;
        errno = 0;
        unsigned long parsed = strtoul(argv[1], &end, 0);
        if (errno != 0 || end == argv[1] || *end != '\0' || parsed == 0) {
            fprintf(stderr, "invalid sample count: %s\n", argv[1]);
            return 2;
        }
        samples = parsed;
    }

    uint64_t *lat = calloc(samples, sizeof(*lat));
    if (lat == NULL) {
        perror("calloc");
        return 2;
    }

    for (size_t i = 0; i < samples; i++) {
        uint64_t start = now_ns();
        do_fixed_work();
        uint64_t end = now_ns();
        lat[i] = end - start;
    }

    qsort(lat, samples, sizeof(*lat), cmp_u64);

    size_t i50 = samples * 50 / 100;
    size_t i99 = samples * 99 / 100;
    size_t i999 = samples * 999 / 1000;
    if (i50 >= samples) i50 = samples - 1;
    if (i99 >= samples) i99 = samples - 1;
    if (i999 >= samples) i999 = samples - 1;

    printf("samples=%zu\n", samples);
    printf("p50_ns=%" PRIu64 "\n", lat[i50]);
    printf("p99_ns=%" PRIu64 "\n", lat[i99]);
    printf("p999_ns=%" PRIu64 "\n", lat[i999]);

    free(lat);
    return 0;
}
