/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_truncate_fsync";
    const char *path = "/tmp/abi_truncate_fsync";
    int fd = open(path, O_CREAT | O_RDWR | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open");
    if (write(fd, "abcdef", 6) != 6)
        return abi_errno_fail(name, "write");
    if (fsync(fd) != 0)
        return abi_errno_fail(name, "fsync");
    if (ftruncate(fd, 3) != 0)
        return abi_errno_fail(name, "ftruncate");
    struct stat st;
    if (fstat(fd, &st) != 0)
        return abi_errno_fail(name, "fstat");
    close(fd);
    unlink(path);
    if (st.st_size != 3)
        return abi_fail(name, "size=%ld", (long)st.st_size);
    return abi_pass(name);
}
