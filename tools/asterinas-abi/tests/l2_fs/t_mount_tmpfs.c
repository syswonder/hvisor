/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/mount.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_mount_tmpfs";
    const char *dir = "/tmp/abi_tmpfs_mount";
    if (geteuid() != 0)
        return abi_skip(name, "requires root");
    if (mkdir(dir, 0755) != 0 && errno != EEXIST)
        return abi_errno_fail(name, "mkdir");
    if (mount("tmpfs", dir, "tmpfs", 0, "size=1m") != 0) {
        if (errno == ENODEV)
            return abi_skip(name, "tmpfs mount unavailable: %s", strerror(errno));
        int s = abi_skip_if_unsupported(name, errno, 1); /* EPERM, ENOSYS, ... */
        if (s)
            return s;
        return abi_errno_fail(name, "mount tmpfs");
    }
    if (umount(dir) != 0)
        return abi_errno_fail(name, "umount");
    rmdir(dir);
    return abi_pass(name);
}
