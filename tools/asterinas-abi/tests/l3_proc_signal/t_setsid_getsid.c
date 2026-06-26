/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_setsid_getsid";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        pid_t sid = setsid();
        if (sid < 0)
            _exit(2);
        if (getsid(0) != sid)
            _exit(3);
        _exit(0);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "status=%d", status);
    return abi_pass(name);
}
