/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include <errno.h>
#include <fcntl.h>
#include <sched.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

static void try_mkdir(const char *path)
{
    if (mkdir(path, 0755) != 0 && errno != EEXIST) {
        printf("mkdir %s failed: %s\n", path, strerror(errno));
    }
}

int main(void)
{
    try_mkdir("/dev");
    try_mkdir("/proc");
    try_mkdir("/sys");
    try_mkdir("/tmp");

    int console = open("/dev/console", O_RDWR);
    if (console >= 0) {
        dup2(console, STDIN_FILENO);
        dup2(console, STDOUT_FILENO);
        dup2(console, STDERR_FILENO);
        if (console > STDERR_FILENO) {
            close(console);
        }
    }

    printf("aster-hv minimal init started\n");
    printf("pid=%d\n", getpid());
    fflush(stdout);

    for (;;) {
        struct timespec ts = {
            .tv_sec = 60,
            .tv_nsec = 0,
        };
        nanosleep(&ts, NULL);
        printf("aster-hv minimal init alive\n");
        fflush(stdout);
    }
}
