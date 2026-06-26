/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_stat_chmod";
    const char *path = "/tmp/abi_stat_chmod";
    int fd = open(path, O_CREAT | O_WRONLY | O_TRUNC, 0600);
    if (fd < 0)
        return abi_errno_fail(name, "open");
    close(fd);
    if (chmod(path, 0644) != 0)
        return abi_errno_fail(name, "chmod");
    struct stat st;
    if (stat(path, &st) != 0)
        return abi_errno_fail(name, "stat");
    unlink(path);
    if ((st.st_mode & 0777) != 0644)
        return abi_fail(name, "mode=%o", st.st_mode & 0777);
    return abi_pass(name);
}
