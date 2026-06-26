/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_access_unlink";
    const char *path = "/tmp/abi_access_unlink";
    int fd = open(path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open");
    if (close(fd) != 0)
        return abi_errno_fail(name, "close");
    if (access(path, F_OK) != 0)
        return abi_errno_fail(name, "access existing");
    if (unlink(path) != 0)
        return abi_errno_fail(name, "unlink");
    if (access(path, F_OK) == 0 || errno != ENOENT)
        return abi_fail(name, "access missing did not report ENOENT");
    return abi_pass(name);
}
