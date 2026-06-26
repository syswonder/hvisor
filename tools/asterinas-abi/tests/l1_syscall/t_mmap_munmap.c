/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/mman.h>

int main(void)
{
    const char *name = "t_mmap_munmap";
    size_t len = 8192;
    char *p = mmap(NULL, len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (p == MAP_FAILED)
        return abi_errno_fail(name, "mmap");
    p[0] = 'x';
    p[len - 1] = 'y';
    if (mprotect(p, len, PROT_READ) != 0)
        return abi_errno_fail(name, "mprotect");
    if (p[0] != 'x' || p[len - 1] != 'y')
        return abi_fail(name, "memory contents changed");
    if (munmap(p, len) != 0)
        return abi_errno_fail(name, "munmap");
    return abi_pass(name);
}
