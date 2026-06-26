/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_proc_self_status";
    FILE *f = fopen("/proc/self/status", "r");
    if (!f) {
        if (errno == ENOENT)
            return abi_skip(name, "procfs unavailable");
        return abi_errno_fail(name, "fopen status");
    }
    char line[256];
    int saw_name = 0;
    while (fgets(line, sizeof(line), f)) {
        if (strncmp(line, "Name:", 5) == 0)
            saw_name = 1;
    }
    fclose(f);
    if (!saw_name)
        return abi_fail(name, "Name field missing");
    return abi_pass(name);
}
