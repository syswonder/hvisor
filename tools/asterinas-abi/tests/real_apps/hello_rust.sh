#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -u
if [ -x /usr/bin/hello_rust ]; then
    bin=/usr/bin/hello_rust
elif [ -x ./hello_rust ]; then
    bin=./hello_rust
else
    echo "hello_rust: SKIP: binary not present"
    exit 77
fi

out=/tmp/abi_hello_rust.out
# Check the binary's own exit status, not grep's: a crashing binary that still
# happens to print "hello" before dying must not pass.
"$bin" >"$out" 2>&1
rc=$?
if [ "$rc" -ne 0 ]; then
    cat "$out"
    echo "hello_rust: FAIL: binary exited rc=$rc"
    exit 1
fi
if grep -q 'hello' "$out"; then
    echo "hello_rust: PASS"
    exit 0
fi
cat "$out"
echo "hello_rust: FAIL: 'hello' not in output"
exit 1
