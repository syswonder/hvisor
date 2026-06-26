/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/prctl.h>

int main(void)
{
    const char *name = "t_prctl_name";
    const char proc_name[] = "abi-prctl";
    char out[32] = {0};
    if (prctl(PR_SET_NAME, proc_name, 0, 0, 0) != 0)
        return abi_errno_fail(name, "PR_SET_NAME");
    if (prctl(PR_GET_NAME, out, 0, 0, 0) != 0)
        return abi_errno_fail(name, "PR_GET_NAME");
    if (strcmp(out, proc_name) != 0)
        return abi_fail(name, "name=%s", out);
    return abi_pass(name);
}
