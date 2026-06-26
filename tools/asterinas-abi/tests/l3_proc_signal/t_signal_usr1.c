/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <signal.h>

static volatile sig_atomic_t seen;

static void on_usr1(int signo)
{
    (void)signo;
    seen = 1;
}

int main(void)
{
    const char *name = "t_signal_usr1";
    if (signal(SIGUSR1, on_usr1) == SIG_ERR)
        return abi_errno_fail(name, "signal");
    if (raise(SIGUSR1) != 0)
        return abi_errno_fail(name, "raise");
    if (!seen)
        return abi_fail(name, "handler not called");
    return abi_pass(name);
}
