/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_pipe2_nonblock";
    int fds[2];
    if (pipe2(fds, O_NONBLOCK | O_CLOEXEC) != 0)
        return abi_errno_fail(name, "pipe2");
    char c;
    ssize_t n = read(fds[0], &c, 1);
    if (n != -1 || (errno != EAGAIN && errno != EWOULDBLOCK))
        return abi_fail(name, "read returned %zd errno=%d", n, errno);
    close(fds[0]);
    close(fds[1]);
    return abi_pass(name);
}
