/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_proc_self_maps";
    FILE *f = fopen("/proc/self/maps", "r");
    if (!f) {
        if (errno == ENOENT)
            return abi_skip(name, "procfs maps unavailable");
        return abi_errno_fail(name, "fopen maps");
    }
    /* Each line is "start-end perms offset dev inode [path]"; require at least
     * one mapping whose permissions column carries the executable bit (the
     * process's own text segment must be executable). Matching '-' alone is
     * tautological since every range line contains it. */
    char line[512];
    int saw_exec = 0;
    while (fgets(line, sizeof(line), f)) {
        char addr[128], perms[16];
        if (sscanf(line, "%127s %15s", addr, perms) == 2) {
            if (strchr(perms, 'x') != NULL) {
                saw_exec = 1;
                break;
            }
        }
    }
    fclose(f);
    if (!saw_exec)
        return abi_fail(name, "no executable mapping found");
    return abi_pass(name);
}
