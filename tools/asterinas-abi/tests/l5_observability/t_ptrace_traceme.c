/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <signal.h>
#include <sys/ptrace.h>
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_ptrace_traceme";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        if (ptrace(PTRACE_TRACEME, 0, NULL, NULL) != 0)
            _exit(abi_errno_is_unsupported(errno, 1) ? 77 : 2);
        raise(SIGSTOP);
        _exit(0);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid stop");
    if (WIFEXITED(status) && WEXITSTATUS(status) == 77)
        return abi_skip(name, "ptrace traceme unavailable");
    if (!WIFSTOPPED(status))
        return abi_fail(name, "child status=%d", status);
    if (ptrace(PTRACE_CONT, pid, NULL, NULL) != 0)
        return abi_errno_fail(name, "ptrace cont");
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid exit");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "exit status=%d", status);
    return abi_pass(name);
}
