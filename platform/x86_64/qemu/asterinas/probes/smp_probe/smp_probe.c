/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include "../common.h"

#include <pthread.h>
#include <stdatomic.h>

struct worker_arg {
    int logical_cpu;
    long iterations;
    atomic_ulong *counters;
};

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--threads N] [--iterations N]\n"
            "\n"
            "Bind one worker per logical CPU, report APIC IDs and per-CPU counters.\n",
            argv0);
}

static void *worker_main(void *opaque)
{
    struct worker_arg *arg = opaque;
    if (bind_to_cpu(arg->logical_cpu) != 0) {
        die_errno("sched_setaffinity");
    }

    uint32_t apic = cpuid_apic_id();
    for (long i = 0; i < arg->iterations; i++) {
        atomic_fetch_add_explicit(&arg->counters[arg->logical_cpu], 1, memory_order_relaxed);
    }

    printf("cpu=%d apic_id=%" PRIu32 " counter=%lu\n",
           arg->logical_cpu,
           apic,
           atomic_load_explicit(&arg->counters[arg->logical_cpu], memory_order_relaxed));
    return NULL;
}

int main(int argc, char **argv)
{
    int ncpu = online_cpu_count();
    int threads = ncpu;
    long iterations = 10000000;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--threads") == 0 && i + 1 < argc) {
            threads = (int)parse_long_arg("--threads", argv[++i], 1, ncpu);
        } else if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc) {
            iterations = parse_long_arg("--iterations", argv[++i], 1, LONG_MAX);
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            usage(argv[0]);
            return 0;
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    printf("online_cpus=%d requested_threads=%d iterations_per_thread=%ld\n",
           ncpu,
           threads,
           iterations);

    pthread_t *tids = calloc((size_t)threads, sizeof(*tids));
    struct worker_arg *args = calloc((size_t)threads, sizeof(*args));
    atomic_ulong *counters = calloc((size_t)ncpu, sizeof(*counters));
    if (!tids || !args || !counters) {
        die_errno("calloc");
    }

    for (int i = 0; i < threads; i++) {
        args[i].logical_cpu = i;
        args[i].iterations = iterations;
        args[i].counters = counters;
        int err = pthread_create(&tids[i], NULL, worker_main, &args[i]);
        if (err != 0) {
            errno = err;
            die_errno("pthread_create");
        }
    }

    for (int i = 0; i < threads; i++) {
        int err = pthread_join(tids[i], NULL);
        if (err != 0) {
            errno = err;
            die_errno("pthread_join");
        }
    }

    int stuck = 0;
    for (int i = 0; i < threads; i++) {
        unsigned long counter = atomic_load_explicit(&counters[i], memory_order_relaxed);
        if (counter != (unsigned long)iterations) {
            fprintf(stderr, "counter mismatch cpu=%d got=%lu expected=%ld\n", i, counter, iterations);
            stuck++;
        }
    }

    free(tids);
    free(args);
    free(counters);
    return stuck == 0 ? 0 : 1;
}
