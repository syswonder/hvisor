#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
#
# Layered ABI test runner. Executes every staged *.test / *.sh case under
# $ABI_TEST_ROOT in L1->L5->App order and writes a machine-readable JSON
# document to $ABI_RESULT_JSON.
#
# Design goals for unattended runs inside a guest:
#   * print one human-readable progress line per case to stdout (live visibility
#     on the serial console, and used by the host to detect hangs);
#   * enforce a per-case wall-clock timeout so a single blocking syscall cannot
#     stall the whole matrix -- using a race-free watchdog (no PID reuse hazard);
#   * never dump the raw JSON to stdout (the init script hex-frames the file
#     instead, which is immune to kernel-log interleaving on the console).
set -u

TEST_ROOT="${ABI_TEST_ROOT:-/usr/bin/abi-tests}"
OUT="${ABI_RESULT_JSON:-/tmp/test_results.json}"
TMP_DIR="${ABI_TMPDIR:-/tmp/abi-run}"
TIMEOUT="${ABI_TEST_TIMEOUT:-45}"
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR" "$(dirname "$OUT")"

now_ms() {
    v="$(date +%s%3N 2>/dev/null || true)"
    case "$v" in
        *N*|"") echo "$(date +%s)000" ;;
        *) echo "$v" ;;
    esac
}

# Race-free per-case timeout. We rely only on primitives that Asterinas handles
# reliably: background a job, `wait <pid>` to read its real exit code, and a
# detached watchdog that polls `kill -0` and force-kills on timeout. Timeout is
# signalled through a per-call flag file keyed by a monotonically increasing
# counter (NOT by PID), so a recycled PID can never resurrect a stale verdict.
# The function sets the global RC and prints nothing (no command-substitution
# subshell, which behaved inconsistently under the guest).
WD_SEQ=0
RC=0
run_with_timeout() {
    logf="$1"; shift
    WD_SEQ=$((WD_SEQ + 1))
    flag="$TMP_DIR/timedout.$WD_SEQ"
    rm -f "$flag"

    "$@" >"$logf" 2>&1 &
    cmd_pid=$!

    (
        i=0
        while [ "$i" -lt "$TIMEOUT" ]; do
            kill -0 "$cmd_pid" 2>/dev/null || exit 0
            sleep 1
            i=$((i + 1))
        done
        : > "$flag"
        kill -9 "$cmd_pid" 2>/dev/null
    ) &
    wd_pid=$!

    wait "$cmd_pid" 2>/dev/null
    RC=$?
    kill -9 "$wd_pid" 2>/dev/null
    wait "$wd_pid" 2>/dev/null
    if [ -f "$flag" ]; then
        RC=124
        rm -f "$flag"
    fi
}

# Emit a captured-output file as a JSON string body (no surrounding quotes).
# Byte-accurate and equivalent to harness/abi_runner.c's json_escape_file:
# escape \ and ", map the common control bytes to short escapes, and escape
# every other byte < 0x20 (and 0x7f) as \u00XX. Working from od's decimal byte
# stream avoids awk's text/locale record handling so arbitrary control bytes in
# captured output can never produce invalid JSON.
json_escape_file() {
    [ -s "$1" ] || return 0
    od -An -v -tu1 "$1" | LC_ALL=C awk '
        BEGIN { ORS="" }
        {
            for (i = 1; i <= NF; i++) {
                b = $i + 0;
                if (b == 92)       printf "\\\\";
                else if (b == 34)  printf "\\\"";
                else if (b == 10)  printf "\\n";
                else if (b == 13)  printf "\\r";
                else if (b == 9)   printf "\\t";
                else if (b < 32 || b == 127) printf "\\u%04x", b;
                else printf "%c", b;
            }
        }'
}

layer_of() {
    case "$1" in
        */l1_syscall/*) echo "L1" ;;
        */l2_fs/*) echo "L2" ;;
        */l3_proc_signal/*) echo "L3" ;;
        */l4_net/*) echo "L4" ;;
        */l5_observability/*) echo "L5" ;;
        */real_apps/*) echo "App" ;;
        *) echo "Unknown" ;;
    esac
}

name_of() {
    base="$(basename "$1")"
    base="${base%.test}"
    base="${base%.sh}"
    echo "$base"
}

find_tests() {
    if [ ! -d "$TEST_ROOT" ]; then
        return 0
    fi
    for layer in l1_syscall l2_fs l3_proc_signal l4_net l5_observability real_apps; do
        if [ -d "$TEST_ROOT/$layer" ]; then
            find "$TEST_ROOT/$layer" -type f \( -name '*.test' -o -name '*.sh' \) | sort
        fi
    done
    find "$TEST_ROOT" -type f \( -name '*.test' -o -name '*.sh' \) \
        ! -path "*/l1_syscall/*" ! -path "*/l2_fs/*" ! -path "*/l3_proc_signal/*" \
        ! -path "*/l4_net/*" ! -path "*/l5_observability/*" ! -path "*/real_apps/*" | sort
}

HOST="$(uname -n 2>/dev/null || echo unknown)"
KERNEL="$(uname -srmo 2>/dev/null || echo unknown)"

TEST_LIST="$TMP_DIR/test_list"
find_tests > "$TEST_LIST"
# Count without relying on `wc -l` quirks across busybox builds.
TOTAL=0
while IFS= read -r _l; do
    [ -n "$_l" ] && TOTAL=$((TOTAL + 1))
done < "$TEST_LIST"

{
    printf '{\n'
    printf '  "schema_version": 1,\n'
    printf '  "metadata": {\n'
    printf '    "environment": "%s",\n' "${ABI_ENV_ID:-unknown}"
    printf '    "measurement_status": "%s",\n' "${ABI_MEASUREMENT_STATUS:-measured}"
    printf '    "test_root": "%s",\n' "$TEST_ROOT"
    printf '    "per_test_timeout_s": %s,\n' "$TIMEOUT"
    printf '    "host": "%s",\n' "$HOST"
    printf '    "kernel": "%s"\n' "$KERNEL"
    printf '  },\n'
    printf '  "results": [\n'
} > "$OUT"

echo "=== ABI run_all: $TOTAL cases, env=${ABI_ENV_ID:-unknown}, per-test timeout=${TIMEOUT}s ==="

first=1
count=0
pass=0; fail=0; skip=0
while IFS= read -r test_path; do
    [ -n "$test_path" ] || continue
    name="$(name_of "$test_path")"
    layer="$(layer_of "$test_path")"
    log="$TMP_DIR/$name.log"
    count=$((count + 1))
    start="$(now_ms)"
    if [ -x "$test_path" ]; then
        run_with_timeout "$log" "$test_path"
    else
        run_with_timeout "$log" sh "$test_path"
    fi
    rc="$RC"
    end="$(now_ms)"
    duration=$((end - start))
    case "$rc" in
        ''|*[!0-9]*) rc=125 ;;
    esac
    status="FAIL"
    if [ "$rc" -eq 0 ]; then status="PASS"; pass=$((pass+1));
    elif [ "$rc" -eq 77 ]; then status="SKIP"; skip=$((skip+1));
    else fail=$((fail+1)); fi
    printf '[%d/%d] %-3s %-22s %-4s rc=%-3d %dms\n' \
        "$count" "$TOTAL" "$layer" "$name" "$status" "$rc" "$duration"
    if [ "$first" -eq 0 ]; then
        printf ',\n' >> "$OUT"
    fi
    first=0
    {
        printf '    {"name":"%s","layer":"%s","status":"%s","exit_code":%s,"duration_ms":%s,"output":"' \
            "$name" "$layer" "$status" "$rc" "$duration"
        json_escape_file "$log"
        printf '"}'
    } >> "$OUT"
done < "$TEST_LIST"

{
    printf '\n  ],\n'
    printf '  "summary": {"total": %s, "pass": %s, "fail": %s, "skip": %s}\n' \
        "$count" "$pass" "$fail" "$skip"
    printf '}\n'
} >> "$OUT"

echo "=== ABI run_all done: total=$count pass=$pass fail=$fail skip=$skip ==="
echo "=== results written to $OUT ==="
