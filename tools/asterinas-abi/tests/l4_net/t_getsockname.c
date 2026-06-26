/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>

int main(void)
{
    const char *name = "t_getsockname";
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0)
        return abi_errno_fail(name, "socket");
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
    close(fd);
    if (ntohs(addr.sin_port) == 0)
        return abi_fail(name, "ephemeral port not assigned");
    return abi_pass(name);
}
