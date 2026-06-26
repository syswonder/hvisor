/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <elf.h>
#include <fcntl.h>

int main(void)
{
    const char *name = "t_auxv_random";
    int fd = open("/proc/self/auxv", O_RDONLY);
    if (fd < 0) {
        if (errno == ENOENT)
            return abi_skip(name, "auxv unavailable");
        return abi_errno_fail(name, "open auxv");
    }
    Elf64_auxv_t ent;
    int saw_random = 0;
    while (read(fd, &ent, sizeof(ent)) == (ssize_t)sizeof(ent)) {
        if (ent.a_type == AT_RANDOM)
            saw_random = 1;
        if (ent.a_type == AT_NULL)
            break;
    }
    close(fd);
    if (!saw_random)
        return abi_fail(name, "AT_RANDOM missing");
    return abi_pass(name);
}
