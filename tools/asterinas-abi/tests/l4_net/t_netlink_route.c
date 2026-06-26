/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <linux/netlink.h>
#include <sys/socket.h>

int main(void)
{
    const char *name = "t_netlink_route";
    int fd = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);
    if (fd < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "socket netlink");
    }
    close(fd);
    return abi_pass(name);
}
