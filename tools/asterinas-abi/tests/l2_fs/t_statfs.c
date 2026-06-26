/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/vfs.h>

int main(void)
{
    const char *name = "t_statfs";
    struct statfs s;
    if (statfs("/tmp", &s) != 0)
        return abi_errno_fail(name, "statfs /tmp");
    /* Block size must be a positive power of two. Deliberately do NOT assert
     * f_type: its value is filesystem-specific and not part of the portable
     * ABI contract we are measuring. */
    /* f_blocks is intentionally not asserted: a size-less filesystem (ramfs,
     * an initramfs rootfs) legitimately reports zero blocks, which is not an
     * ABI violation. */
    unsigned long long bsize = (unsigned long long)s.f_bsize;
    if (bsize == 0 || (bsize & (bsize - 1)) != 0)
        return abi_fail(name, "f_bsize not a power of two: %llu", bsize);
    return abi_pass(name);
}
