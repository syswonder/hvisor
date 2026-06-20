/* SPDX-License-Identifier: MulanPSL-2.0 */
/* Copyright (c) 2026 hvisor contributors */

/*
 * probe_selftest: a self-contained guest /init that exercises every SMP/RT
 * probe and prints a human-readable summary to the console. It is intended to
 * be the PID 1 of a tiny initramfs so that booting the Asterinas zone1 image
 * immediately demonstrates, with no shell required:
 *
 *   - the CPU topology the guest sees (smp_probe: online CPUs + APIC IDs);
 *   - context-switch throughput (ctxsw);
 *   - periodic-timer wakeup jitter (timer_jitter);
 *   - cross-core round-trip latency (ipi_pingpong).
 *
 * Each probe is executed as a child process whose stdout is captured. Bare
 * numeric lines (raw samples) are reduced to count/min/max/mean; header and
 * key-value lines are echoed verbatim. The full raw streams are also left in
 * /tmp so they can be copied out and fed to pipeline/stats.py for percentiles.
 *
 * NOTE: when this image is booted under QEMU TCG (no KVM), the numbers are
 * valid only as a boot check and MUST NOT be used for performance conclusions.
 */

#define _GNU_SOURCE

#include <fcntl.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static void try_mount(const char *src, const char *tgt, const char *fstype)
{
    (void)mkdir(tgt, 0755);
    if (mount(src, tgt, fstype, 0, NULL) != 0) {
        /* Best effort; Asterinas may auto-populate /dev or lack a given fs. */
    }
}

static void attach_console(void)
{
    int fd = open("/dev/console", O_RDWR);
    if (fd < 0) {
        fd = open("/dev/ttyS0", O_RDWR);
    }
    if (fd >= 0) {
        dup2(fd, STDIN_FILENO);
        dup2(fd, STDOUT_FILENO);
        dup2(fd, STDERR_FILENO);
        if (fd > STDERR_FILENO) {
            close(fd);
        }
    }
}

static int is_blank(const char *s)
{
    for (; *s; s++) {
        if (*s != ' ' && *s != '\t' && *s != '\r' && *s != '\n') {
            return 0;
        }
    }
    return 1;
}

static int cmp_double(const void *a, const void *b)
{
    double x = *(const double *)a;
    double y = *(const double *)b;
    return (x > y) - (x < y);
}

/* Linear-interpolation percentile over an ascending-sorted array, matching
 * numpy.percentile (and thus pipeline/stats.py) on the same raw stream. */
static double percentile(const double *sorted, long n, double p)
{
    if (n <= 0) {
        return 0.0;
    }
    if (n == 1) {
        return sorted[0];
    }
    double rank = p / 100.0 * (double)(n - 1);
    long lo = (long)rank;
    double frac = rank - (double)lo;
    if (lo + 1 < n) {
        return sorted[lo] + frac * (sorted[lo + 1] - sorted[lo]);
    }
    return sorted[lo];
}

/* Summarise one captured probe stream: echo header/key-value lines verbatim and
 * reduce bare numeric sample lines to count/min/percentiles/max/mean. The full
 * raw stream is also left under /tmp for offline analysis with pipeline/stats.py. */
static void summarise(const char *path)
{
    FILE *f = fopen(path, "r");
    if (!f) {
        printf("  (could not reopen %s)\n", path);
        return;
    }
    char line[512];
    double *samples = NULL;
    long n = 0;
    long cap = 0;
    double sum = 0;
    while (fgets(line, sizeof(line), f)) {
        if (is_blank(line)) {
            continue;
        }
        char *end = NULL;
        double v = strtod(line, &end);
        /* Treat as a sample only if the whole line is one number. */
        while (end && (*end == ' ' || *end == '\t' || *end == '\r' || *end == '\n')) {
            end++;
        }
        if (end && *end == '\0' && end != line && isfinite(v)) {
            if (n == cap) {
                cap = cap ? cap * 2 : 1024;
                double *grown = realloc(samples, (size_t)cap * sizeof(*samples));
                if (!grown) {
                    free(samples);
                    fclose(f);
                    printf("  (out of memory summarising %s)\n", path);
                    return;
                }
                samples = grown;
            }
            samples[n++] = v;
            sum += v;
        } else {
            /* Header (#...) or key=value line: echo it. */
            size_t len = strlen(line);
            if (len && line[len - 1] == '\n') {
                line[len - 1] = '\0';
            }
            printf("  %s\n", line);
        }
    }
    fclose(f);
    if (n > 0) {
        qsort(samples, (size_t)n, sizeof(*samples), cmp_double);
        printf("  [samples] count=%ld min=%.2f p50=%.2f p90=%.2f p99=%.2f p99.9=%.2f "
               "max=%.2f mean=%.2f\n",
               n, samples[0], percentile(samples, n, 50.0), percentile(samples, n, 90.0),
               percentile(samples, n, 99.0), percentile(samples, n, 99.9), samples[n - 1],
               sum / (double)n);
    }
    free(samples);
    fflush(stdout);
}

/* Run argv[0] with stdout redirected into a capture file, then summarise it. */
static int run_probe(const char *title, char *const argv[], const char *capture)
{
    printf("\n=== PROBE: %s ===\n", title);
    fflush(stdout);

    pid_t pid = fork();
    if (pid < 0) {
        printf("  fork failed\n");
        return -1;
    }
    if (pid == 0) {
        int out = open(capture, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (out >= 0) {
            dup2(out, STDOUT_FILENO);
            if (out > STDOUT_FILENO) {
                close(out);
            }
        }
        execv(argv[0], argv);
        /* execv only returns on failure. */
        _exit(127);
    }

    int status = 0;
    if (waitpid(pid, &status, 0) < 0) {
        printf("  waitpid failed\n");
        return -1;
    }
    if (!WIFEXITED(status)) {
        printf("  probe terminated abnormally (status=%d)\n", status);
    } else if (WEXITSTATUS(status) != 0) {
        printf("  probe exit code=%d\n", WEXITSTATUS(status));
    }
    summarise(capture);
    return WIFEXITED(status) ? WEXITSTATUS(status) : -1;
}

int main(void)
{
    attach_console();
    try_mount("proc", "/proc", "proc");
    try_mount("sysfs", "/sys", "sysfs");
    try_mount("devtmpfs", "/dev", "devtmpfs");
    try_mount("tmpfs", "/tmp", "tmpfs");

    printf("\n");
    printf("############################################################\n");
    printf("# aster-hv-smp guest probe self-test (PID %d)\n", getpid());
    printf("# Asterinas-on-hvisor SMP/RT functional smoke\n");
    printf("############################################################\n");
    fflush(stdout);

    long ncpu = sysconf(_SC_NPROCESSORS_ONLN);
    if (ncpu < 1) {
        ncpu = 1;
    }
    char ncpu_str[24];
    snprintf(ncpu_str, sizeof(ncpu_str), "%ld", ncpu);

    /* Probes live in /bin inside the guest initramfs; PROBE_DIR overrides this
     * so the launcher can also be exercised from a host test tree. */
    const char *dir = getenv("PROBE_DIR");
    if (!dir || !*dir) {
        dir = "/bin";
    }
    char smp_path[256], ctxsw_path[256], jitter_path[256], ipi_path[256];
    snprintf(smp_path, sizeof(smp_path), "%s/smp_probe", dir);
    snprintf(ctxsw_path, sizeof(ctxsw_path), "%s/ctxsw", dir);
    snprintf(jitter_path, sizeof(jitter_path), "%s/timer_jitter", dir);
    snprintf(ipi_path, sizeof(ipi_path), "%s/ipi_pingpong", dir);

    {
        char *argv[] = {smp_path, "--threads", ncpu_str,
                        "--iterations", "2000000", NULL};
        run_probe("smp_probe (topology + APIC IDs)", argv, "/tmp/smp.txt");
    }
    {
        char *argv[] = {ctxsw_path, "--iterations", "20000", NULL};
        run_probe("ctxsw (context-switch rate)", argv, "/tmp/ctxsw.txt");
    }
    {
        char *argv[] = {jitter_path, "--period-us", "1000",
                        "--duration", "2", NULL};
        run_probe("timer_jitter (1ms period, 2s)", argv, "/tmp/jitter.txt");
    }
    if (ncpu > 1) {
        char *argv[] = {ipi_path, "--iterations", "20000", NULL};
        run_probe("ipi_pingpong (cross-core RTT)", argv, "/tmp/ipi.txt");
    } else {
        printf("\n=== PROBE: ipi_pingpong ===\n");
        printf("  skipped: needs >= 2 online CPUs\n");
    }

    printf("\n");
    printf("############################################################\n");
    printf("# aster-hv-smp guest probe self-test COMPLETE\n");
    printf("# raw streams: /tmp/{smp,ctxsw,jitter,ipi}.txt\n");
    printf("############################################################\n");
    fflush(stdout);

    for (;;) {
        struct timespec ts = {.tv_sec = 3600, .tv_nsec = 0};
        nanosleep(&ts, NULL);
    }
    return 0;
}
