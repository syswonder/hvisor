/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <signal.h>
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_kill_signal";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        pause();
        _exit(1);
    }
    usleep(100000);
    if (kill(pid, SIGTERM) != 0)
        return abi_errno_fail(name, "kill");
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid");
    if (!WIFSIGNALED(status) || WTERMSIG(status) != SIGTERM)
        return abi_fail(name, "status=%d", status);
    return abi_pass(name);
}
