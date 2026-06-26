/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_mkdir_rename_unlink";
    const char *dir = "/tmp/abi_mkdir_rename";
    char a[128], b[128];
    if (mkdir(dir, 0755) != 0 && errno != EEXIST)
        return abi_errno_fail(name, "mkdir");
    snprintf(a, sizeof(a), "%s/a", dir);
    snprintf(b, sizeof(b), "%s/b", dir);
    int fd = open(a, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open");
    close(fd);
    if (rename(a, b) != 0)
        return abi_errno_fail(name, "rename");
    if (unlink(b) != 0)
        return abi_errno_fail(name, "unlink");
    rmdir(dir);
    return abi_pass(name);
}
