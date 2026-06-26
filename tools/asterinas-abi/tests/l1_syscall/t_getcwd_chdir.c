/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <limits.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_getcwd_chdir";
    char old[PATH_MAX];
    if (!getcwd(old, sizeof(old)))
        return abi_errno_fail(name, "getcwd old");
    if (mkdir("/tmp/abi_getcwd_dir", 0755) != 0 && errno != EEXIST)
        return abi_errno_fail(name, "mkdir");
    if (chdir("/tmp/abi_getcwd_dir") != 0)
        return abi_errno_fail(name, "chdir");
    char now[PATH_MAX];
    if (!getcwd(now, sizeof(now)))
        return abi_errno_fail(name, "getcwd now");
    if (strstr(now, "abi_getcwd_dir") == NULL)
        return abi_fail(name, "unexpected cwd %s", now);
    if (chdir(old) != 0)
        return abi_errno_fail(name, "restore cwd");
    rmdir("/tmp/abi_getcwd_dir");
    return abi_pass(name);
}
