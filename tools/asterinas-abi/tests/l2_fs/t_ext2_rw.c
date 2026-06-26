/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/mount.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_ext2_rw";
    const char *mountpoint = "/tmp/abi_ext2_mount";
    const char *dev = getenv("ABI_EXT2_DEVICE");
    static const char payload[] = "ext2-probe\n";
    if (!dev || dev[0] == '\0')
        return abi_skip(name, "ABI_EXT2_DEVICE not set");
    if (geteuid() != 0)
        return abi_skip(name, "requires root");
    if (mkdir(mountpoint, 0755) != 0 && errno != EEXIST)
        return abi_errno_fail(name, "mkdir mountpoint");
    if (mount(dev, mountpoint, "ext2", 0, "") != 0) {
        if (errno == EPERM || errno == ENODEV || errno == ENOSYS || errno == EACCES)
            return abi_skip(name, "ext2 mount unavailable: %s", strerror(errno));
        return abi_errno_fail(name, "mount ext2");
    }

    char path[160];
    snprintf(path, sizeof(path), "%s/probe", mountpoint);

    /* Write, then durably flush so the read-back exercises the real fs. */
    FILE *wf = fopen(path, "w");
    if (!wf) {
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fopen write");
    }
    if (fputs(payload, wf) == EOF) {
        fclose(wf);
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fputs");
    }
    if (fflush(wf) != 0 || fsync(fileno(wf)) != 0) {
        fclose(wf);
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fsync");
    }
    if (fclose(wf) != 0) {
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fclose write");
    }

    /* Re-open and read the content back. */
    char buf[sizeof(payload)] = {0};
    FILE *rf = fopen(path, "r");
    if (!rf) {
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fopen read");
    }
    size_t got = fread(buf, 1, sizeof(payload) - 1, rf);
    int read_err = ferror(rf);
    if (fclose(rf) != 0) {
        umount(mountpoint);
        rmdir(mountpoint);
        return abi_errno_fail(name, "fclose read");
    }

    int verify_fail = (read_err != 0) || got != sizeof(payload) - 1 ||
                      memcmp(buf, payload, sizeof(payload) - 1) != 0;

    if (umount(mountpoint) != 0)
        return abi_errno_fail(name, "umount");
    rmdir(mountpoint);

    if (verify_fail)
        return abi_fail(name, "read-back mismatch: got=%zu '%s'", got, buf);
    return abi_pass(name);
}
