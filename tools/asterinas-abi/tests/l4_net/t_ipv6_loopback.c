/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_ipv6_loopback";
    int srv = socket(AF_INET6, SOCK_STREAM, 0);
    if (srv < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "socket");
    }
    int one = 1;
    setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one));
    struct sockaddr_in6 addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin6_family = AF_INET6;
    addr.sin6_addr = in6addr_loopback;
    addr.sin6_port = 0;
    if (bind(srv, (struct sockaddr *)&addr, sizeof(addr)) != 0)
        return abi_errno_fail(name, "bind");
    if (listen(srv, 1) != 0)
        return abi_errno_fail(name, "listen");
    socklen_t len = sizeof(addr);
    if (getsockname(srv, (struct sockaddr *)&addr, &len) != 0)
        return abi_errno_fail(name, "getsockname");
    /* Bound the accept()/read() so a stalled peer becomes a fast specific FAIL
     * instead of tripping the harness watchdog. */
    struct timeval tv = {.tv_sec = 5, .tv_usec = 0};
    setsockopt(srv, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        int cli = socket(AF_INET6, SOCK_STREAM, 0);
        if (cli < 0)
            _exit(2);
        setsockopt(cli, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
        if (connect(cli, (struct sockaddr *)&addr, sizeof(addr)) != 0)
            _exit(3);
        if (write(cli, "6", 1) != 1)
            _exit(4);
        close(cli);
        _exit(0);
    }
    int acc = accept(srv, NULL, NULL);
    if (acc < 0)
        return abi_errno_fail(name, "accept");
    setsockopt(acc, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    char c = 0;
    ssize_t n = read(acc, &c, 1);
    close(acc);
    close(srv);
    int status = 0;
    waitpid(pid, &status, 0);
    if (n != 1 || c != '6' || !WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "n=%zd c=%c status=%d", n, c, status);
    return abi_pass(name);
}
