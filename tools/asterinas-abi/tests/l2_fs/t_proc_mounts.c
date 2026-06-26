/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_proc_mounts";
    FILE *f = fopen("/proc/mounts", "r");
    if (!f) {
        if (errno == ENOENT)
            return abi_skip(name, "/proc/mounts unavailable");
        return abi_errno_fail(name, "fopen /proc/mounts");
    }
    char line[256];
    int ok = fgets(line, sizeof(line), f) != NULL;
    fclose(f);
    if (!ok)
        return abi_fail(name, "/proc/mounts empty");
    return abi_pass(name);
}
