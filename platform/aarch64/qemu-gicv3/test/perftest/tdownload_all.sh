#!/bin/bash
# perftest wrapper for aarch64/qemu-gicv3
set -e

WORKSPACE_ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
"${WORKSPACE_ROOT}/platform/aarch64/qemu-gicv3/test/systemtest/tdownload_all.sh"
