/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"

int main(void)
{
    const char *name = "t_tmpfile_rw";
    FILE *f = tmpfile();
    if (!f)
        return abi_errno_fail(name, "tmpfile");
    fputs("tmpfile-data", f);
    rewind(f);
    char buf[32] = {0};
    if (!fgets(buf, sizeof(buf), f)) {
        fclose(f);
        return abi_fail(name, "fgets failed");
    }
    fclose(f);
    if (strcmp(buf, "tmpfile-data") != 0)
        return abi_fail(name, "bad data %s", buf);
    return abi_pass(name);
}
