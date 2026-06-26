/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_brk_sbrk";
    errno = 0;
    void *base = sbrk(0);
    if (base == (void *)-1) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "sbrk base");
    }
    errno = 0;
    if (sbrk(4096) == (void *)-1) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "sbrk grow");
    }
    char *grown = (char *)base;
    grown[0] = 'a';
    grown[4095] = 'z';
    if (brk(base) != 0)
        return abi_errno_fail(name, "brk restore");
    return abi_pass(name);
}
