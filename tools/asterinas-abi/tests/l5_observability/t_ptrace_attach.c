/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <signal.h>
#include <sys/ptrace.h>
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_ptrace_attach";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        for (;;)
            pause();
    }
    usleep(100000);
    if (ptrace(PTRACE_ATTACH, pid, NULL, NULL) != 0) {
        int saved = errno;
        kill(pid, SIGKILL);
        waitpid(pid, NULL, 0);
        if (saved == EPERM || saved == ENOSYS)
            return abi_skip(name, "ptrace attach unavailable: %s", strerror(saved));
        errno = saved;
        return abi_errno_fail(name, "ptrace attach");
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "wait stopped child");
    if (!WIFSTOPPED(status))
        return abi_fail(name, "child not stopped");
    if (ptrace(PTRACE_DETACH, pid, NULL, NULL) != 0)
        return abi_errno_fail(name, "ptrace detach");
    kill(pid, SIGTERM);
    waitpid(pid, NULL, 0);
    return abi_pass(name);
}
