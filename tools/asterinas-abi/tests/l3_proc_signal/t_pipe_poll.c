/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <poll.h>

int main(void)
{
    const char *name = "t_pipe_poll";
    int fds[2];
    if (pipe(fds) != 0)
        return abi_errno_fail(name, "pipe");
    if (write(fds[1], "x", 1) != 1)
        return abi_errno_fail(name, "write");
    struct pollfd pfd = {.fd = fds[0], .events = POLLIN};
    int rc = poll(&pfd, 1, 1000);
    close(fds[0]);
    close(fds[1]);
    if (rc != 1 || !(pfd.revents & POLLIN))
        return abi_fail(name, "poll rc=%d revents=%x", rc, pfd.revents);
    return abi_pass(name);
}
