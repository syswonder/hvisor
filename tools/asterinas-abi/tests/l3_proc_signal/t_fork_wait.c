/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_fork_wait";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0)
        _exit(12);
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 12)
        return abi_fail(name, "status=%d", status);
    return abi_pass(name);
}
