#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -u
if ! command -v gdb >/dev/null 2>&1; then
    echo "gdb_batch: SKIP: gdb not installed"
    exit 77
fi
target="${ABI_GDB_TARGET:-/bin/true}"
out=/tmp/abi_gdb_batch.out
gdb -nx -q -batch -ex "file $target" -ex "info files" >"$out" 2>&1
rc=$?
# Exit status alone is not sufficient: gdb can return 0 while having failed to
# load the target. Require the "info files" report to actually describe the
# loaded ELF (its entry point and .text section).
if [ "$rc" -eq 0 ] && grep -q 'Entry point:' "$out" && grep -q '\.text' "$out"; then
    echo "gdb_batch: PASS"
    exit 0
fi
cat "$out"
echo "gdb_batch: FAIL: rc=$rc or missing expected 'info files' tokens"
exit 1
