/* SPDX-License-Identifier: MulanPSL-2.0 */
#ifndef FAULT_PROBE_H
#define FAULT_PROBE_H

#include <errno.h>
#include <inttypes.h>
#include <setjmp.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#if defined(__GNUC__) || defined(__clang__)
#define FAULT_PROBE_UNUSED __attribute__((unused))
#else
#define FAULT_PROBE_UNUSED
#endif

static jmp_buf fault_probe_jmp;
static volatile sig_atomic_t fault_probe_signal = 0;

static void fault_probe_handler(int sig)
{
    fault_probe_signal = sig;
    longjmp(fault_probe_jmp, 1);
}

static int fault_probe_install_handlers(void)
{
    signal(SIGSEGV, fault_probe_handler);
    signal(SIGBUS, fault_probe_handler);
    signal(SIGILL, fault_probe_handler);
    return 0;
}

static uintptr_t fault_probe_parse_addr(int argc, char **argv, uintptr_t fallback)
{
    if (argc < 2) {
        return fallback;
    }

    errno = 0;
    char *end = NULL;
    unsigned long long parsed = strtoull(argv[1], &end, 0);
    if (errno != 0 || end == argv[1] || *end != '\0') {
        fprintf(stderr, "invalid address %s\n", argv[1]);
        exit(2);
    }
    return (uintptr_t)parsed;
}

static int FAULT_PROBE_UNUSED fault_probe_read_addr(uintptr_t addr, const char *label)
{
    fault_probe_signal = 0;
    if (fault_probe_install_handlers() != 0) {
        return 2;
    }

    if (setjmp(fault_probe_jmp) == 0) {
        volatile uint8_t *p = (volatile uint8_t *)addr;
        uint8_t v = *p;
        printf("FAIL: %s read from 0x%" PRIxPTR " succeeded, value=%u\n", label, addr, v);
        return 1;
    }

    printf("PASS: %s access to 0x%" PRIxPTR " trapped with signal %d\n",
           label, addr, fault_probe_signal);
    return 0;
}

static int FAULT_PROBE_UNUSED fault_probe_write_addr(uintptr_t addr, const char *label)
{
    fault_probe_signal = 0;
    if (fault_probe_install_handlers() != 0) {
        return 2;
    }

    if (setjmp(fault_probe_jmp) == 0) {
        volatile uint8_t *p = (volatile uint8_t *)addr;
        *p = 0xa5;
        printf("FAIL: %s write to 0x%" PRIxPTR " succeeded\n", label, addr);
        return 1;
    }

    printf("PASS: %s write to 0x%" PRIxPTR " trapped with signal %d\n",
           label, addr, fault_probe_signal);
    return 0;
}

#endif
