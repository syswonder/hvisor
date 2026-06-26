/* SPDX-License-Identifier: MulanPSL-2.0 */
#ifndef ABI_TEST_COMMON_H
#define ABI_TEST_COMMON_H

#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define ABI_SKIP_CODE 77
#if defined(__GNUC__)
#define ABI_UNUSED __attribute__((unused))
#else
#define ABI_UNUSED
#endif

static ABI_UNUSED int abi_pass(const char *name)
{
    printf("%s: PASS\n", name);
    return 0;
}

static ABI_UNUSED int abi_fail(const char *name, const char *fmt, ...)
{
    va_list ap;
    fprintf(stderr, "%s: FAIL: ", name);
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
    fputc('\n', stderr);
    return 1;
}

static ABI_UNUSED int abi_skip(const char *name, const char *fmt, ...)
{
    va_list ap;
    printf("%s: SKIP: ", name);
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
    putchar('\n');
    return ABI_SKIP_CODE;
}

static ABI_UNUSED int abi_errno_fail(const char *name, const char *what)
{
    return abi_fail(name, "%s: errno=%d (%s)", what, errno, strerror(errno));
}

/*
 * Return non-zero when `err` indicates the optional primitive is simply not
 * available in this environment (so the case should SKIP, exit 77, rather than
 * inflate the differential as a FAIL). Pass include_eperm=1 only for primitives
 * whose *creation/availability* legitimately fails with EPERM when the feature
 * is disabled or unprivileged (e.g. ptrace, mount). This must only be consulted
 * on the creation/availability errno of the optional primitive; real assertion
 * failures must stay FAIL.
 */
static ABI_UNUSED int abi_errno_is_unsupported(int err, int include_eperm)
{
    switch (err) {
    case ENOSYS:
#ifdef EOPNOTSUPP
    case EOPNOTSUPP:
#endif
#if defined(ENOTSUP) && (!defined(EOPNOTSUPP) || ENOTSUP != EOPNOTSUPP)
    case ENOTSUP:
#endif
#ifdef EAFNOSUPPORT
    case EAFNOSUPPORT:
#endif
#ifdef EPROTONOSUPPORT
    case EPROTONOSUPPORT:
#endif
        return 1;
    default:
        break;
    }
    if (include_eperm && err == EPERM)
        return 1;
    return 0;
}

/*
 * If `err` marks the optional primitive `name` exercises as unavailable, emit a
 * SKIP and return ABI_SKIP_CODE; otherwise return 0 so the caller can proceed
 * to its normal error handling. Usage:
 *     if (fd < 0) {
 *         int s = abi_skip_if_unsupported(name, errno, 0);
 *         if (s) return s;
 *         return abi_errno_fail(name, "socket");
 *     }
 */
static ABI_UNUSED int abi_skip_if_unsupported(const char *name, int err, int include_eperm)
{
    if (abi_errno_is_unsupported(err, include_eperm))
        return abi_skip(name, "primitive unsupported here: errno=%d (%s)",
                        err, strerror(err));
    return 0;
}

static ABI_UNUSED const char *abi_self_path(const char *argv0)
{
    static char path[512];
    ssize_t n = readlink("/proc/self/exe", path, sizeof(path) - 1);
    if (n > 0) {
        path[n] = '\0';
        return path;
    }
    return argv0;
}

#endif
