/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_open_close";
    int fd = open("/dev/null", O_RDONLY);
    if (fd < 0)
        return abi_errno_fail(name, "open /dev/null");
    if (close(fd) != 0)
        return abi_errno_fail(name, "close");
    return abi_pass(name);
}
