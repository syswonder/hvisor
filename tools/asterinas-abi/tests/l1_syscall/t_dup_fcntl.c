/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_dup_fcntl";
    int fd = open("/dev/null", O_WRONLY);
    if (fd < 0)
        return abi_errno_fail(name, "open /dev/null");
    int dupfd = dup(fd);
    if (dupfd < 0)
        return abi_errno_fail(name, "dup");
    int flags = fcntl(dupfd, F_GETFD);
    if (flags < 0)
        return abi_errno_fail(name, "fcntl getfd");
    if (fcntl(dupfd, F_SETFD, flags | FD_CLOEXEC) != 0)
        return abi_errno_fail(name, "fcntl setfd");
    int fixed = dup2(fd, 99);
    if (fixed != 99)
        return abi_errno_fail(name, "dup2");
    close(99);
    close(dupfd);
    close(fd);
    return abi_pass(name);
}
