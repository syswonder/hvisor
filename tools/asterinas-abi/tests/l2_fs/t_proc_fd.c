/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <fcntl.h>

int main(void)
{
    const char *name = "t_proc_fd";
    int fd = open("/dev/null", O_RDONLY);
    if (fd < 0)
        return abi_errno_fail(name, "open /dev/null");
    char link[64], target[256];
    snprintf(link, sizeof(link), "/proc/self/fd/%d", fd);
    ssize_t n = readlink(link, target, sizeof(target) - 1);
    close(fd);
    if (n < 0) {
        if (errno == ENOENT)
            return abi_skip(name, "procfs fd entries unavailable");
        return abi_errno_fail(name, "readlink proc fd");
    }
    target[n] = '\0';
    /* The fd was opened on /dev/null; the procfs symlink must resolve to
     * exactly that path, not merely contain the substring "null". */
    if (strcmp(target, "/dev/null") != 0)
        return abi_fail(name, "unexpected target %s", target);
    return abi_pass(name);
}
