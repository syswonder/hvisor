/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <signal.h>

static volatile sig_atomic_t delivered;

static void handler(int signo)
{
    (void)signo;
    delivered++;
}

int main(void)
{
    const char *name = "t_sigaction_mask";
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = handler;
    if (sigaction(SIGUSR1, &sa, NULL) != 0)
        return abi_errno_fail(name, "sigaction");
    sigset_t set;
    sigemptyset(&set);
    sigaddset(&set, SIGUSR1);
    if (sigprocmask(SIG_BLOCK, &set, NULL) != 0)
        return abi_errno_fail(name, "sigprocmask block");
    if (kill(getpid(), SIGUSR1) != 0)
        return abi_errno_fail(name, "kill");
    if (delivered != 0)
        return abi_fail(name, "signal delivered while blocked");
    if (sigprocmask(SIG_UNBLOCK, &set, NULL) != 0)
        return abi_errno_fail(name, "sigprocmask unblock");
    usleep(100000);
    if (delivered != 1)
        return abi_fail(name, "delivered=%d", delivered);
    return abi_pass(name);
}
