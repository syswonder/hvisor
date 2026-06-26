/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_symlink_readlink";
    const char *target = "/tmp/abi_symlink_target";
    const char *link = "/tmp/abi_symlink_link";
    unlink(link);
    unlink(target);
    FILE *f = fopen(target, "w");
    if (!f)
        return abi_errno_fail(name, "fopen target");
    fputs("x", f);
    fclose(f);
    if (symlink(target, link) != 0)
        return abi_errno_fail(name, "symlink");
    char buf[256];
    ssize_t n = readlink(link, buf, sizeof(buf) - 1);
    if (n < 0)
        return abi_errno_fail(name, "readlink");
    buf[n] = '\0';
    unlink(link);
    unlink(target);
    if (strcmp(buf, target) != 0)
        return abi_fail(name, "target mismatch %s", buf);
    return abi_pass(name);
}
