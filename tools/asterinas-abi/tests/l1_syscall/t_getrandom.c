/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/random.h>

int main(void)
{
    const char *name = "t_getrandom";
    unsigned char buf[16] = {0};
    ssize_t n = getrandom(buf, sizeof(buf), 0);
    if (n < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "getrandom");
    }
    if (n != (ssize_t)sizeof(buf))
        return abi_fail(name, "short getrandom %zd", n);
    unsigned char acc = 0;
    for (size_t i = 0; i < sizeof(buf); i++)
        acc |= buf[i];
    if (acc == 0)
        return abi_fail(name, "all-zero random buffer");
    return abi_pass(name);
}
