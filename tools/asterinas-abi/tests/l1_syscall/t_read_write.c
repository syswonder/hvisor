/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_read_write";
    const char *path = "/tmp/abi_rw";
    const char data[] = "hello-abi\n";
    int fd = open(path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open write");
    if (write(fd, data, strlen(data)) != (ssize_t)strlen(data))
        return abi_errno_fail(name, "write");
    close(fd);
    char buf[64] = {0};
    fd = open(path, O_RDONLY);
    if (fd < 0)
        return abi_errno_fail(name, "open read");
    ssize_t n = read(fd, buf, sizeof(buf));
    close(fd);
    unlink(path);
    if (n < 0)
        return abi_errno_fail(name, "read");
    if (strncmp(buf, data, strlen(data)) != 0)
        return abi_fail(name, "unexpected data '%s'", buf);
    return abi_pass(name);
}
