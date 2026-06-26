/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <sched.h>
#include <signal.h>
#include <sys/wait.h>

static int child_fn(void *arg)
{
    int *value = (int *)arg;
    return *value == 42 ? 0 : 3;
}

int main(void)
{
    const char *name = "t_clone_wait";
    enum { STACK_SIZE = 65536 };
    void *stack = malloc(STACK_SIZE);
    if (!stack)
        return abi_fail(name, "malloc stack failed");
    int value = 42;
    pid_t pid = clone(child_fn, (char *)stack + STACK_SIZE, SIGCHLD, &value);
    if (pid < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s) {
            free(stack);
            return s;
        }
        free(stack);
        return abi_errno_fail(name, "clone");
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid) {
        free(stack);
        return abi_errno_fail(name, "waitpid");
    }
    free(stack);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "child status=%d", status);
    return abi_pass(name);
}
