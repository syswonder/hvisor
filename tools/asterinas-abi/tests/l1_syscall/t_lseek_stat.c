/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_lseek_stat";
    const char *path = "/tmp/abi_lseek_stat";
    int fd = open(path, O_CREAT | O_RDWR | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open");
    if (write(fd, "abcdef", 6) != 6)
        return abi_errno_fail(name, "write");
    if (lseek(fd, 2, SEEK_SET) != 2)
        return abi_errno_fail(name, "lseek");
    char c = 0;
    if (read(fd, &c, 1) != 1 || c != 'c')
        return abi_fail(name, "read after seek got %c", c);
    struct stat st;
    if (fstat(fd, &st) != 0)
        return abi_errno_fail(name, "fstat");
    close(fd);
    unlink(path);
    if (st.st_size != 6)
        return abi_fail(name, "size=%ld", (long)st.st_size);
    return abi_pass(name);
}
