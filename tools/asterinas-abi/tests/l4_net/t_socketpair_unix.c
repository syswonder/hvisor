/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/socket.h>

int main(void)
{
    const char *name = "t_socketpair_unix";
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv) != 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "socketpair");
    }
    if (write(sv[0], "u", 1) != 1)
        return abi_errno_fail(name, "write");
    char c = 0;
    if (read(sv[1], &c, 1) != 1)
        return abi_errno_fail(name, "read");
    close(sv[0]);
    close(sv[1]);
    if (c != 'u')
        return abi_fail(name, "bad char");
    return abi_pass(name);
}
