/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/time.h>
#include <time.h>

int main(void)
{
    const char *name = "t_gettimeofday_clock";
    struct timeval tv;
    struct timespec ts;
    if (gettimeofday(&tv, NULL) != 0)
        return abi_errno_fail(name, "gettimeofday");
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0)
        return abi_errno_fail(name, "clock_gettime monotonic");
    if (tv.tv_sec <= 0 || ts.tv_sec < 0)
        return abi_fail(name, "invalid time values");
    return abi_pass(name);
}
