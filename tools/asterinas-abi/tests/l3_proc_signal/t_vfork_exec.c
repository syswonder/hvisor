/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/wait.h>

int main(int argc, char **argv)
{
    const char *name = "t_vfork_exec";
    if (argc > 1 && strcmp(argv[1], "--child") == 0)
        _exit(0);
    pid_t pid = vfork();
    if (pid < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s)
            return s;
        return abi_errno_fail(name, "vfork");
    }
    if (pid == 0) {
        char *const child_argv[] = {(char *)abi_self_path(argv[0]), (char *)"--child", NULL};
        execv(child_argv[0], child_argv);
        _exit(127);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "status=%d", status);
    return abi_pass(name);
}
