/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

#include "../common.h"

#include <limits.h>
#include <sys/wait.h>

static void usage(const char *argv0)
{
    fprintf(stderr,
            "usage: %s [--iterations N]\n"
            "\n"
            "Parent and child ping-pong one byte through pipes and report context switches/s.\n",
            argv0);
}

static void read_exact(int fd, char *buf, size_t len)
{
    size_t done = 0;
    while (done < len) {
        ssize_t n = read(fd, buf + done, len - done);
        if (n < 0) {
            die_errno("read");
        }
        if (n == 0) {
            fprintf(stderr, "unexpected EOF\n");
            exit(EXIT_FAILURE);
        }
        done += (size_t)n;
    }
}

static void write_exact(int fd, const char *buf, size_t len)
{
    size_t done = 0;
    while (done < len) {
        ssize_t n = write(fd, buf + done, len - done);
        if (n < 0) {
            die_errno("write");
        }
        done += (size_t)n;
    }
}

int main(int argc, char **argv)
{
    long iterations = 100000;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--iterations") == 0 && i + 1 < argc) {
            iterations = parse_long_arg("--iterations", argv[++i], 1, LONG_MAX / 2);
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            usage(argv[0]);
            return 0;
        } else {
            usage(argv[0]);
            return 2;
        }
    }

    int parent_to_child[2];
    int child_to_parent[2];
    if (pipe(parent_to_child) != 0 || pipe(child_to_parent) != 0) {
        die_errno("pipe");
    }

    pid_t pid = fork();
    if (pid < 0) {
        die_errno("fork");
    }

    char token = 'x';
    if (pid == 0) {
        close(parent_to_child[1]);
        close(child_to_parent[0]);
        for (long i = 0; i < iterations; i++) {
            read_exact(parent_to_child[0], &token, 1);
            write_exact(child_to_parent[1], &token, 1);
        }
        close(parent_to_child[0]);
        close(child_to_parent[1]);
        return 0;
    }

    close(parent_to_child[0]);
    close(child_to_parent[1]);
    uint64_t t0 = nsec_now_monotonic();
    for (long i = 0; i < iterations; i++) {
        write_exact(parent_to_child[1], &token, 1);
        read_exact(child_to_parent[0], &token, 1);
    }
    uint64_t t1 = nsec_now_monotonic();
    close(parent_to_child[1]);
    close(child_to_parent[0]);

    int status = 0;
    if (waitpid(pid, &status, 0) < 0) {
        die_errno("waitpid");
    }
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        fprintf(stderr, "child failed with status=%d\n", status);
        return 1;
    }

    double elapsed_s = (double)(t1 - t0) / 1000000000.0;
    double switches = (double)iterations * 2.0;
    printf("iterations=%ld elapsed_s=%.9f ctxsw_per_s=%.2f\n",
           iterations,
           elapsed_s,
           switches / elapsed_s);
    return 0;
}
