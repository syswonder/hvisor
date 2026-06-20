/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */
#ifndef ASTER_HV_SMP_COMMON_H
#define ASTER_HV_SMP_COMMON_H

#define _GNU_SOURCE

#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <sched.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static inline void die_errno(const char *msg)
{
    int err = errno;
    fprintf(stderr, "%s: %s\n", msg, strerror(err));
    exit(EXIT_FAILURE);
}

static inline long parse_long_arg(const char *name, const char *value, long min, long max)
{
    char *end = NULL;
    errno = 0;
    long parsed = strtol(value, &end, 10);
    if (errno != 0 || end == value || *end != '\0' || parsed < min || parsed > max) {
        fprintf(stderr, "invalid %s: %s (expected %ld..%ld)\n", name, value, min, max);
        exit(EXIT_FAILURE);
    }
    return parsed;
}

static inline uint64_t nsec_now_monotonic(void)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        die_errno("clock_gettime(CLOCK_MONOTONIC)");
    }
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

static inline void add_nsec(struct timespec *ts, int64_t ns)
{
    ts->tv_nsec += ns;
    while (ts->tv_nsec >= 1000000000L) {
        ts->tv_nsec -= 1000000000L;
        ts->tv_sec++;
    }
    while (ts->tv_nsec < 0) {
        ts->tv_nsec += 1000000000L;
        ts->tv_sec--;
    }
}

static inline int bind_to_cpu(int cpu)
{
    cpu_set_t set;
    CPU_ZERO(&set);
    CPU_SET(cpu, &set);
    return sched_setaffinity(0, sizeof(set), &set);
}

static inline int online_cpu_count(void)
{
    long ncpu = sysconf(_SC_NPROCESSORS_ONLN);
    if (ncpu <= 0 || ncpu > 4096) {
        fprintf(stderr, "unexpected online CPU count: %ld\n", ncpu);
        exit(EXIT_FAILURE);
    }
    return (int)ncpu;
}

static inline uint64_t rdtsc_ordered(void)
{
    uint32_t lo;
    uint32_t hi;
#if defined(__x86_64__) || defined(__i386__)
    __asm__ __volatile__("lfence\n\t"
                         "rdtsc"
                         : "=a"(lo), "=d"(hi)
                         :
                         : "memory");
    return ((uint64_t)hi << 32) | lo;
#else
    return nsec_now_monotonic();
#endif
}

static inline uint32_t cpuid_apic_id(void)
{
#if defined(__x86_64__) || defined(__i386__)
    uint32_t eax = 0;
    uint32_t ebx = 0;
    uint32_t ecx = 0;
    uint32_t edx = 0;

    /* Discover the maximum supported basic CPUID leaf. */
    __asm__ __volatile__("cpuid"
                         : "+a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
                         :
                         : "memory");
    uint32_t max_leaf = eax;

    /* Prefer the x2APIC ID from leaf 0x0B (32-bit, in EDX). hvisor virtualises
     * the LAPIC in x2APIC mode, so leaf 1's 8-bit initial APIC ID can truncate
     * for APIC IDs >= 256; leaf 0x0B reports the full x2APIC ID. */
    if (max_leaf >= 0x0B) {
        eax = 0x0B;
        ecx = 0;
        __asm__ __volatile__("cpuid"
                             : "+a"(eax), "=b"(ebx), "+c"(ecx), "=d"(edx)
                             :
                             : "memory");
        /* EBX[15:0] != 0 indicates leaf 0x0B enumerates a valid level. */
        if ((ebx & 0xffffu) != 0) {
            return edx;
        }
    }

    /* Fallback: legacy 8-bit initial APIC ID from leaf 1, EBX[31:24]. */
    eax = 1;
    ebx = 0;
    ecx = 0;
    edx = 0;
    __asm__ __volatile__("cpuid"
                         : "+a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
                         :
                         : "memory");
    return (ebx >> 24) & 0xffu;
#else
    return UINT32_MAX;
#endif
}

#endif
