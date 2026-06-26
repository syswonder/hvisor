/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <arpa/inet.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <poll.h>
#include <sys/socket.h>

int main(void)
{
    const char *name = "t_nonblock_connect";
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "socket");
    }
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags < 0 || fcntl(fd, F_SETFL, flags | O_NONBLOCK) != 0) {
        close(fd);
        return abi_errno_fail(name, "fcntl nonblock");
    }
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = htons(9); /* discard port, expected to have no listener */

    int rc = connect(fd, (struct sockaddr *)&addr, sizeof(addr));
    if (rc == 0) {
        /* A nonblocking connect to a dead port must not succeed instantly. */
        close(fd);
        return abi_fail(name, "connect succeeded immediately to a dead port");
    }
    if (errno == ECONNREFUSED) {
        /* Some stacks deliver the RST synchronously: a valid immediate refusal. */
        close(fd);
        return abi_pass(name);
    }
    if (errno != EINPROGRESS) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s) {
            close(fd);
            return s;
        }
        int saved = errno;
        close(fd);
        errno = saved;
        return abi_errno_fail(name, "connect");
    }

    /* In-progress: wait for writability, then inspect SO_ERROR. */
    struct pollfd pf = {.fd = fd, .events = POLLOUT};
    int pr;
    do {
        pr = poll(&pf, 1, 5000);
    } while (pr < 0 && errno == EINTR);
    if (pr < 0) {
        close(fd);
        return abi_errno_fail(name, "poll");
    }
    if (pr == 0) {
        close(fd);
        return abi_fail(name, "connect did not complete within timeout");
    }

    int soerr = 0;
    socklen_t len = sizeof(soerr);
    if (getsockopt(fd, SOL_SOCKET, SO_ERROR, &soerr, &len) != 0) {
        close(fd);
        return abi_errno_fail(name, "getsockopt SO_ERROR");
    }
    close(fd);
    /* For a dead discard port we expect a deterministic connection error; the
     * point of this case is the EINPROGRESS->poll->SO_ERROR handshake, so any
     * resolved verdict (refused, or unexpectedly succeeded) is acceptable as
     * long as the nonblocking machinery worked. A poll-readable socket whose
     * SO_ERROR is still EINPROGRESS would be a real ABI bug. */
    if (soerr == EINPROGRESS)
        return abi_fail(name, "socket writable but SO_ERROR still EINPROGRESS");
    return abi_pass(name);
}
