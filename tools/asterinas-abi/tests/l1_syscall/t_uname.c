/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/utsname.h>

int main(void)
{
    const char *name = "t_uname";
    struct utsname u;
    if (uname(&u) != 0)
        return abi_errno_fail(name, "uname");
    if (u.sysname[0] == '\0' || u.machine[0] == '\0')
        return abi_fail(name, "empty uname fields");
    printf("sysname=%s release=%s machine=%s\n", u.sysname, u.release, u.machine);
    return abi_pass(name);
}
