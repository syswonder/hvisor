#!/bin/bash
set -eu

cargo clean
cargo build --target aarch64-unknown-linux-gnu
qemu-system-aarch64 \
    -M virt \
    -m 1024M \
    -cpu cortex-a53 \
    -nographic \
    -machine virtualization=on \
    -kernel target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust