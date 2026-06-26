/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/ioctl.h>

int main(void)
{
    const char *name = "t_ioctl_fionread";
    int fds[2];
    if (pipe(fds) != 0)
        return abi_errno_fail(name, "pipe");
    const char msg[] = "abcde";
    if (write(fds[1], msg, sizeof(msg)) != (ssize_t)sizeof(msg))
        return abi_errno_fail(name, "write");
    int available = 0;
    if (ioctl(fds[0], FIONREAD, &available) != 0)
        return abi_errno_fail(name, "ioctl FIONREAD");
    close(fds[0]);
    close(fds[1]);
    if (available < (int)sizeof(msg))
        return abi_fail(name, "available=%d", available);
    return abi_pass(name);
}
