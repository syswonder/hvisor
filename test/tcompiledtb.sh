#!/bin/bash
set -e  # Exit immediately if any command fails

# Compile device tree in a subshell to maintain working directory
(
    cd images/aarch64/devicetree && 
    make all
)
# Subshell automatically returns to original directory after execution