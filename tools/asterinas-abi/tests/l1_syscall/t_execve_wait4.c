/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/resource.h>
#include <sys/wait.h>

int main(int argc, char **argv)
{
    const char *name = "t_execve_wait4";
    if (argc > 1 && strcmp(argv[1], "--child") == 0)
        return 0;

    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        char *const child_argv[] = {(char *)abi_self_path(argv[0]), (char *)"--child", NULL};
        char *const child_env[] = {(char *)"PATH=/bin:/usr/bin", NULL};
        execve(child_argv[0], child_argv, child_env);
        _exit(127);
    }
    int status = 0;
    struct rusage ru;
    if (wait4(pid, &status, 0, &ru) != pid)
        return abi_errno_fail(name, "wait4");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "unexpected child status=%d", status);
    return abi_pass(name);
}
