/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include "../common.h"

#include <limits.h>
#include <pthread.h>
#include <stdatomic.h>

struct shared_state {
    atomic_long request;
    atomic_long ack;
    long iterations;
    int cpu_a;
    int cpu_b;
    uint64_t *latencies;
};

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--iterations N] [--cpu-a CPU] [--cpu-b CPU] [--ns]\n"
            "\n"
            "Two pinned threads exchange an atomic flag and print one latency per line.\n"
            "Default unit is TSC cycles on x86; --ns uses CLOCK_MONOTONIC deltas.\n",
            argv0);
}

static bool output_ns;

static inline uint64_t timestamp(void)
{
    return output_ns ? nsec_now_monotonic() : rdtsc_ordered();
}

static void cpu_relax(void)
{
#if defined(__x86_64__) || defined(__i386__)
    __asm__ __volatile__("pause" ::: "memory");
#endif
}

static void *ponger_main(void *opaque)
{
    struct shared_state *state = opaque;
    if (bind_to_cpu(state->cpu_b) != 0) {
        die_errno("sched_setaffinity ponger");
    }

    for (long i = 1; i <= state->iterations; i++) {
        while (atomic_load_explicit(&state->request, memory_order_acquire) < i) {
            cpu_relax();
        }
        atomic_store_explicit(&state->ack, i, memory_order_release);
    }
    return NULL;
}

int main(int argc, char **argv)
{
    int ncpu = online_cpu_count();
    struct shared_state state = {
        .request = ATOMIC_VAR_INIT(0),
        .ack = ATOMIC_VAR_INIT(0),
        .iterations = 100000,
        .cpu_a = 0,
        .cpu_b = ncpu > 1 ? 1 : 0,
        .latencies = NULL,
    };

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc) {
            state.iterations = parse_long_arg("--iterations", argv[++i], 1, LONG_MAX);
        } else if (strcmp(argv[i], "--cpu-a") == 0 && i + 1 < argc) {
            state.cpu_a = (int)parse_long_arg("--cpu-a", argv[++i], 0, ncpu - 1);
        } else if (strcmp(argv[i], "--cpu-b") == 0 && i + 1 < argc) {
            state.cpu_b = (int)parse_long_arg("--cpu-b", argv[++i], 0, ncpu - 1);
        } else if (strcmp(argv[i], "--ns") == 0) {
            output_ns = true;
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            usage(argv[0]);
            return 0;
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    if (state.cpu_a == state.cpu_b) {
        fprintf(stderr, "cpu-a and cpu-b must differ\n");
        return 2;
    }

    state.latencies = calloc((size_t)state.iterations, sizeof(*state.latencies));
    if (!state.latencies) {
        die_errno("calloc latencies");
    }

    pthread_t ponger;
    int err = pthread_create(&ponger, NULL, ponger_main, &state);
    if (err != 0) {
        errno = err;
        die_errno("pthread_create");
    }

    if (bind_to_cpu(state.cpu_a) != 0) {
        die_errno("sched_setaffinity pinger");
    }

    for (long i = 1; i <= state.iterations; i++) {
        uint64_t t0 = timestamp();
        atomic_store_explicit(&state.request, i, memory_order_release);
        while (atomic_load_explicit(&state.ack, memory_order_acquire) < i) {
            cpu_relax();
        }
        uint64_t t1 = timestamp();
        state.latencies[i - 1] = t1 - t0;
    }

    err = pthread_join(ponger, NULL);
    if (err != 0) {
        errno = err;
        die_errno("pthread_join");
    }

    printf("# unit=%s iterations=%ld cpu_a=%d cpu_b=%d\n",
           output_ns ? "ns" : "cycles",
           state.iterations,
           state.cpu_a,
           state.cpu_b);
    for (long i = 0; i < state.iterations; i++) {
        printf("%" PRIu64 "\n", state.latencies[i]);
    }

    free(state.latencies);
    return 0;
}
