#!/bin/bash
set -e -x  # Exit immediately if any command fails

# Compile device tree in a subshell to maintain working directory
(
    cd platform/riscv64/qemu-plic/image/dts
    make all
)
# Subshell automatically returns to original directory after execution