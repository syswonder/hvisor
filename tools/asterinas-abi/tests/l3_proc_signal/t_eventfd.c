/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <stdint.h>
#include <sys/eventfd.h>

int main(void)
{
    const char *name = "t_eventfd";
    int fd = eventfd(0, 0);
    if (fd < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "eventfd");
    }
    uint64_t one = 1, out = 0;
    if (write(fd, &one, sizeof(one)) != (ssize_t)sizeof(one))
        return abi_errno_fail(name, "write eventfd");
    if (read(fd, &out, sizeof(out)) != (ssize_t)sizeof(out))
        return abi_errno_fail(name, "read eventfd");
    close(fd);
    if (out != 1)
        return abi_fail(name, "counter=%llu", (unsigned long long)out);
    return abi_pass(name);
}
