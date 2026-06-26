/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_getpid_getppid";
    pid_t pid = getpid();
    pid_t ppid = getppid();
    if (pid <= 0 || ppid <= 0)
        return abi_fail(name, "bad pid=%ld ppid=%ld", (long)pid, (long)ppid);

    /* Cross-check the relationship: a child's getppid() must equal this
     * process's getpid(). This catches a getppid() that merely returns a
     * plausible non-zero constant. */
    pid_t child = fork();
    if (child < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "fork");
    }
    if (child == 0)
        _exit(getppid() == pid ? 0 : 1);

    int status = 0;
    if (waitpid(child, &status, 0) != child)
        return abi_errno_fail(name, "waitpid");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "child getppid mismatch (parent pid=%ld) status=%d",
                        (long)pid, status);
    return abi_pass(name);
}
