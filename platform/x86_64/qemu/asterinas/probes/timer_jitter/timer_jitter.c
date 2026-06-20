/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include "../common.h"

#include <limits.h>

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--period-us N] [--duration SEC] [--cpu CPU]\n"
            "\n"
            "Measure absolute CLOCK_MONOTONIC wakeup jitter and print one ns sample per line.\n",
            argv0);
}

static int64_t timespec_delta_ns(const struct timespec *a, const struct timespec *b)
{
    int64_t sec = (int64_t)a->tv_sec - (int64_t)b->tv_sec;
    int64_t nsec = (int64_t)a->tv_nsec - (int64_t)b->tv_nsec;
    return sec * 1000000000ll + nsec;
}

int main(int argc, char **argv)
{
    int ncpu = online_cpu_count();
    long period_us = 1000;
    long duration_s = 60;
    int cpu = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--period-us") == 0 && i + 1 < argc) {
            period_us = parse_long_arg("--period-us", argv[++i], 1, 10000000);
        } else if (strcmp(argv[i], "--duration") == 0 && i + 1 < argc) {
            duration_s = parse_long_arg("--duration", argv[++i], 1, LONG_MAX / 1000000);
        } else if (strcmp(argv[i], "--cpu") == 0 && i + 1 < argc) {
            cpu = (int)parse_long_arg("--cpu", argv[++i], 0, ncpu - 1);
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            usage(argv[0]);
            return 0;
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    if (cpu >= 0 && bind_to_cpu(cpu) != 0) {
        die_errno("sched_setaffinity");
    }

    long samples = (duration_s * 1000000L) / period_us;
    if (samples <= 0) {
        samples = 1;
    }

    struct timespec next;
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &next) != 0) {
        die_errno("clock_gettime");
    }
    add_nsec(&next, period_us * 1000L);

    printf("# unit=ns period_us=%ld duration_s=%ld samples=%ld cpu=%d\n",
           period_us,
           duration_s,
           samples,
           cpu);
    for (long i = 0; i < samples; i++) {
        int rc;
        do {
            rc = clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, &next, NULL);
        } while (rc == EINTR);
        if (rc != 0) {
            errno = rc;
            die_errno("clock_nanosleep");
        }

        if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
            die_errno("clock_gettime");
        }
        printf("%" PRId64 "\n", timespec_delta_ns(&now, &next));
        add_nsec(&next, period_us * 1000L);
    }

    return 0;
}
