/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <linux/futex.h>
#include <stdatomic.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <time.h>

static int futex_call(atomic_int *uaddr, int op, int val, const struct timespec *timeout)
{
    return syscall(SYS_futex, uaddr, op, val, timeout, NULL, 0);
}

int main(void)
{
    const char *name = "t_futex_wait_wake";
    atomic_int *shared = mmap(NULL, sizeof(*shared), PROT_READ | PROT_WRITE,
                              MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    if (shared == MAP_FAILED)
        return abi_errno_fail(name, "mmap");
    atomic_store(shared, 0);
    pid_t pid = fork();
    if (pid < 0)
        return abi_errno_fail(name, "fork");
    if (pid == 0) {
        struct timespec ts = {.tv_sec = 5, .tv_nsec = 0};
        while (atomic_load(shared) == 0) {
            int rc = futex_call(shared, FUTEX_WAIT, 0, &ts);
            if (rc != 0) {
                /* The condition is re-checked at the top of the loop, so a
                 * spurious wakeup, signal interruption or timeout is safe to
                 * retry. EAGAIN means the value already changed -- also safe.
                 * ENOSYS means FUTEX is absent: signal the parent to SKIP. */
                if (errno == EINTR || errno == ETIMEDOUT || errno == EAGAIN)
                    continue;
                _exit(errno == ENOSYS ? 77 : 2);
            }
        }
        _exit(0);
    }
    usleep(100000);
    atomic_store(shared, 1);
    if (futex_call(shared, FUTEX_WAKE, 1, NULL) < 0) {
        int s = abi_skip_if_unsupported(name, errno, 0);
        if (s) {
            int status = 0;
            waitpid(pid, &status, 0);
            munmap(shared, sizeof(*shared));
            return s;
        }
        return abi_errno_fail(name, "futex wake");
    }
    int status = 0;
    if (waitpid(pid, &status, 0) != pid)
        return abi_errno_fail(name, "waitpid");
    munmap(shared, sizeof(*shared));
    if (WIFEXITED(status) && WEXITSTATUS(status) == 77)
        return abi_skip(name, "FUTEX_WAIT unsupported");
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0)
        return abi_fail(name, "status=%d", status);
    return abi_pass(name);
}
