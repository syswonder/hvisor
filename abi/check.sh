#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

./abi/generate-header.sh "$tmp"
if ! cmp -s abi/include/hvisor_abi.h "$tmp"; then
    echo "abi/include/hvisor_abi.h is out of date" >&2
    diff -u abi/include/hvisor_abi.h "$tmp" || true
    exit 1
fi

cc -std=c11 -Iabi/include -c abi/tests/abi.c -o /tmp/hvisor_abi_check.o
cargo test -p hvisor-abi
echo "ABI checks passed"
