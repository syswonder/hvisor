/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/time.h>

int main(void)
{
    const char *name = "t_udp_loopback";
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0)
        return abi_errno_fail(name, "socket");
    /* Bound the blocking recvfrom() so a lost datagram fails fast and
     * specifically instead of tripping the harness watchdog. */
    struct timeval tv = {.tv_sec = 5, .tv_usec = 0};
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = 0;
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0)
        return abi_errno_fail(name, "bind");
    socklen_t len = sizeof(addr);
    if (getsockname(fd, (struct sockaddr *)&addr, &len) != 0)
        return abi_errno_fail(name, "getsockname");
    if (sendto(fd, "d", 1, 0, (struct sockaddr *)&addr, sizeof(addr)) != 1)
        return abi_errno_fail(name, "sendto");
    char c = 0;
    ssize_t n = recvfrom(fd, &c, 1, 0, NULL, NULL);
    close(fd);
    if (n != 1 || c != 'd')
        return abi_fail(name, "n=%zd c=%c", n, c);
    return abi_pass(name);
}
