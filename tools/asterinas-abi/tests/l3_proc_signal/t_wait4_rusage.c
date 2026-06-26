/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sys/resource.h>
#include <sys/wait.h>

int main(void)
{
    const char *name = "t_wait4_rusage";
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0)
        _exit(0);
    int status = 0;
    struct rusage ru;
    if (wait4(pid, &status, 0, &ru) != pid)
        return abi_errno_fail(name, "wait4");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "status=%d", status);
    if (ru.ru_utime.tv_sec < 0 || ru.ru_stime.tv_sec < 0)
        return abi_fail(name, "negative rusage");
    return abi_pass(name);
}
