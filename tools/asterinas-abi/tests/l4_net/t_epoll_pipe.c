/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/epoll.h>

int main(void)
{
    const char *name = "t_epoll_pipe";
    int fds[2];
    if (pipe(fds) != 0)
        return abi_errno_fail(name, "pipe");
    int ep = epoll_create1(0);
    if (ep < 0)
        return abi_errno_fail(name, "epoll_create1");
    struct epoll_event ev = {.events = EPOLLIN, .data.fd = fds[0]};
    if (epoll_ctl(ep, EPOLL_CTL_ADD, fds[0], &ev) != 0)
        return abi_errno_fail(name, "epoll_ctl");
    if (write(fds[1], "z", 1) != 1)
        return abi_errno_fail(name, "write");
    struct epoll_event out;
    int rc = epoll_wait(ep, &out, 1, 1000);
    close(ep);
    close(fds[0]);
    close(fds[1]);
    if (rc != 1 || out.data.fd != fds[0])
        return abi_fail(name, "epoll rc=%d", rc);
    return abi_pass(name);
}
