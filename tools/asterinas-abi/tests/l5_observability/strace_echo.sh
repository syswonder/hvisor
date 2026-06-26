#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -u
if ! command -v strace >/dev/null 2>&1; then
    echo "strace_echo: SKIP: strace not installed"
    exit 77
fi
out=/tmp/abi_strace_echo.out
strace -o "$out" /bin/echo hello >/tmp/abi_strace_echo.stdout 2>&1
rc=$?

# strace relies on ptrace; when ptrace is unavailable/blocked the case is not
# applicable rather than a real failure.
if grep -Eq 'ptrace|Operation not permitted' "$out" /tmp/abi_strace_echo.stdout 2>/dev/null; then
    echo "strace_echo: SKIP: ptrace unavailable"
    exit 77
fi
if [ "$rc" -ne 0 ]; then
    cat /tmp/abi_strace_echo.stdout
    echo "strace_echo: FAIL"
    exit 1
fi

# Require real syscall trace lines (anchored to the line start so they cannot
# match arbitrary substrings) including the write that emits "hello".
if grep -Eq '^execve\(' "$out" && grep -Eq '^write\(.*hello' "$out"; then
    echo "strace_echo: PASS"
    exit 0
fi
echo "strace_echo: FAIL: expected execve()/write(hello) syscalls not in trace"
exit 1
