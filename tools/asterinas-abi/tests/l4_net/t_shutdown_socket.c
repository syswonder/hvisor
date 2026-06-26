/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/socket.h>

int main(void)
{
    const char *name = "t_shutdown_socket";
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv) != 0)
        return abi_errno_fail(name, "socketpair");
    if (shutdown(sv[0], SHUT_WR) != 0)
        return abi_errno_fail(name, "shutdown");
    char c = 0;
    ssize_t n = read(sv[1], &c, 1);
    close(sv[0]);
    close(sv[1]);
    if (n != 0)
        return abi_fail(name, "expected EOF got %zd", n);
    return abi_pass(name);
}
