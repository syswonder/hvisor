/* SPDX-License-Identifier: MulanPSL-2.0 */
/*
 * abi_runner -- a self-contained, statically linked test driver for the ABI
 * harness. It replaces the shell watchdog inside the guest, where `sleep`,
 * command substitution and nested background jobs proved unreliable on a
 * young kernel. It depends only on primitives the suite itself exercises:
 * fork/execve/waitpid/pipe/poll/kill/dup2/opendir. The per-case timeout uses
 * poll() on a liveness pipe (no sleep/alarm needed): the child inherits the
 * pipe write end across execve, so poll() on the read end returns POLLHUP the
 * instant the child exits; if poll() times out first, we SIGKILL the child.
 *
 * Output: a results.json document (same schema as run_all.sh) plus a live
 * progress line per case on stdout.
 *
 * Env (mirrors run_all.sh):
 *   ABI_TEST_ROOT   (default /usr/bin/abi-tests)
 *   ABI_RESULT_JSON (default /tmp/test_results.json)
 *   ABI_TEST_TIMEOUT seconds (default 45)
 *   ABI_ENV_ID      (default unknown)
 *   ABI_MEASUREMENT_STATUS (default measured)
 */
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define MAX_TESTS 512
#define NAME_MAX_LEN 256
#define PATH_MAX_LEN 1024

static const char *LAYERS[] = {
    "l1_syscall", "l2_fs", "l3_proc_signal", "l4_net", "l5_observability", "real_apps"};
static const char *LAYER_TAGS[] = {"L1", "L2", "L3", "L4", "L5", "App"};
#define NLAYERS (sizeof(LAYERS) / sizeof(LAYERS[0]))

struct test_case {
    char path[PATH_MAX_LEN];
    char name[NAME_MAX_LEN];
    const char *layer_tag;
    int is_script;
};

static struct test_case tests[MAX_TESTS];
static int ntests = 0;

static int cmp_tests(const void *a, const void *b)
{
    return strcmp(((const struct test_case *)a)->name,
                  ((const struct test_case *)b)->name);
}

static void add_test(const char *dir, const char *fname, const char *layer_tag)
{
    size_t len = strlen(fname);
    int is_script;
    if (len > 5 && strcmp(fname + len - 5, ".test") == 0)
        is_script = 0;
    else if (len > 3 && strcmp(fname + len - 3, ".sh") == 0)
        is_script = 1;
    else
        return;
    if (ntests >= MAX_TESTS)
        return;
    struct test_case *t = &tests[ntests];
    snprintf(t->path, sizeof(t->path), "%s/%s", dir, fname);
    /* name = basename without extension */
    size_t cut = is_script ? len - 3 : len - 5;
    if (cut >= sizeof(t->name))
        cut = sizeof(t->name) - 1;
    memcpy(t->name, fname, cut);
    t->name[cut] = '\0';
    t->layer_tag = layer_tag;
    t->is_script = is_script;
    ntests++;
}

static void scan_layer(const char *root, const char *layer, const char *tag)
{
    char dir[PATH_MAX_LEN];
    snprintf(dir, sizeof(dir), "%s/%s", root, layer);
    DIR *d = opendir(dir);
    if (!d)
        return;
    int start = ntests;
    struct dirent *de;
    while ((de = readdir(d)) != NULL) {
        if (de->d_name[0] == '.')
            continue;
        add_test(dir, de->d_name, tag);
    }
    closedir(d);
    /* deterministic order within a layer */
    if (ntests > start)
        qsort(&tests[start], ntests - start, sizeof(struct test_case), cmp_tests);
}

/* Escape a captured-output file into the open JSON stream. */
static void json_escape_file(FILE *out, const char *path)
{
    FILE *f = fopen(path, "rb");
    if (!f)
        return;
    int c;
    while ((c = fgetc(f)) != EOF) {
        switch (c) {
        case '\\': fputs("\\\\", out); break;
        case '"': fputs("\\\"", out); break;
        case '\n': fputs("\\n", out); break;
        case '\r': fputs("\\r", out); break;
        case '\t': fputs("\\t", out); break;
        default:
            if (c < 0x20 || c == 0x7f)
                fprintf(out, "\\u%04x", c);
            else
                fputc(c, out);
        }
    }
    fclose(f);
}

/* Run one case with a poll()-based timeout. Returns exit code, or 124 on
 * timeout, 125 on internal error. Child stdout/stderr go to log_path. */
static int run_one(const struct test_case *t, const char *log_path, int timeout_s)
{
    int pfd[2];
    if (pipe(pfd) != 0)
        return 125;

    pid_t pid = fork();
    if (pid < 0) {
        close(pfd[0]);
        close(pfd[1]);
        return 125;
    }
    if (pid == 0) {
        /* child */
        close(pfd[0]);
        /* keep pfd[1] open across execve as a liveness token */
        int fd = open(log_path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (fd >= 0) {
            dup2(fd, 1);
            dup2(fd, 2);
            if (fd > 2)
                close(fd);
        }
        if (t->is_script) {
            char *argv[] = {(char *)"/bin/sh", (char *)t->path, NULL};
            execv("/bin/sh", argv);
        } else {
            char *argv[] = {(char *)t->path, NULL};
            execv(t->path, argv);
        }
        _exit(127);
    }

    /* parent */
    close(pfd[1]); /* only the child holds the write end now */
    struct pollfd pf;
    pf.fd = pfd[0];
    pf.events = POLLIN;

    int timed_out = 0;
    int poll_err = 0;
    int pr;
    do {
        pr = poll(&pf, 1, timeout_s * 1000);
    } while (pr < 0 && errno == EINTR);
    if (pr == 0) {
        /* timeout: child still alive (write end still open) */
        timed_out = 1;
        kill(pid, SIGKILL);
    } else if (pr < 0) {
        /* unexpected poll() failure: reap the child rather than risk an
         * unbounded waitpid, and report an internal error. */
        poll_err = 1;
        kill(pid, SIGKILL);
    }
    close(pfd[0]);

    int status = 0;
    while (waitpid(pid, &status, 0) < 0 && errno == EINTR)
        ;

    if (poll_err)
        return 125;
    if (timed_out)
        return 124;
    if (WIFEXITED(status))
        return WEXITSTATUS(status);
    if (WIFSIGNALED(status))
        return 128 + WTERMSIG(status);
    return 125;
}

static long now_ms(void)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0)
        return ts.tv_sec * 1000L + ts.tv_nsec / 1000000L;
    return (long)time(NULL) * 1000L;
}

static const char *getenv_def(const char *k, const char *d)
{
    const char *v = getenv(k);
    return (v && *v) ? v : d;
}

int main(void)
{
    const char *root = getenv_def("ABI_TEST_ROOT", "/usr/bin/abi-tests");
    const char *out_path = getenv_def("ABI_RESULT_JSON", "/tmp/test_results.json");
    const char *env_id = getenv_def("ABI_ENV_ID", "unknown");
    const char *mstatus = getenv_def("ABI_MEASUREMENT_STATUS", "measured");
    int timeout_s = atoi(getenv_def("ABI_TEST_TIMEOUT", "45"));
    if (timeout_s <= 0)
        timeout_s = 45;

    for (size_t i = 0; i < NLAYERS; i++)
        scan_layer(root, LAYERS[i], LAYER_TAGS[i]);

    FILE *out = fopen(out_path, "w");
    if (!out) {
        fprintf(stderr, "abi_runner: cannot open %s\n", out_path);
        return 1;
    }

    char host[128] = "unknown";
    gethostname(host, sizeof(host) - 1);

    fprintf(out, "{\n  \"schema_version\": 1,\n  \"metadata\": {\n");
    fprintf(out, "    \"environment\": \"%s\",\n", env_id);
    fprintf(out, "    \"measurement_status\": \"%s\",\n", mstatus);
    fprintf(out, "    \"test_root\": \"%s\",\n", root);
    fprintf(out, "    \"per_test_timeout_s\": %d,\n", timeout_s);
    fprintf(out, "    \"runner\": \"abi_runner\",\n");
    fprintf(out, "    \"host\": \"%s\"\n", host);
    fprintf(out, "  },\n  \"results\": [\n");

    char log_path[PATH_MAX_LEN];
    int pass = 0, fail = 0, skip = 0;

    printf("=== abi_runner: %d cases, env=%s, per-test timeout=%ds ===\n",
           ntests, env_id, timeout_s);
    fflush(stdout);

    for (int i = 0; i < ntests; i++) {
        snprintf(log_path, sizeof(log_path), "/tmp/abi-%s.log", tests[i].name);
        long t0 = now_ms();
        int rc = run_one(&tests[i], log_path, timeout_s);
        long dur = now_ms() - t0;

        const char *status;
        if (rc == 0) { status = "PASS"; pass++; }
        else if (rc == 77) { status = "SKIP"; skip++; }
        else { status = "FAIL"; fail++; }

        printf("[%d/%d] %-3s %-22s %-4s rc=%-3d %ldms\n",
               i + 1, ntests, tests[i].layer_tag, tests[i].name, status, rc, dur);
        fflush(stdout);

        if (i)
            fputs(",\n", out);
        fprintf(out,
                "    {\"name\":\"%s\",\"layer\":\"%s\",\"status\":\"%s\","
                "\"exit_code\":%d,\"duration_ms\":%ld,\"output\":\"",
                tests[i].name, tests[i].layer_tag, status, rc, dur);
        json_escape_file(out, log_path);
        fputs("\"}", out);
        fflush(out);
        unlink(log_path);
    }

    fprintf(out, "\n  ],\n  \"summary\": {\"total\": %d, \"pass\": %d, \"fail\": %d, \"skip\": %d}\n}\n",
            ntests, pass, fail, skip);
    fclose(out);

    printf("=== abi_runner done: total=%d pass=%d fail=%d skip=%d ===\n",
           ntests, pass, fail, skip);
    printf("=== results written to %s ===\n", out_path);
    fflush(stdout);
    return 0;
}
