#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
timeout_s="${HVISOR_QEMU_TIMEOUT:-30}"
log="${HVISOR_QEMU_LOG:-$(mktemp)}"
keep_log=0
if [ -n "${HVISOR_QEMU_LOG:-}" ]; then
    keep_log=1
fi

cleanup() {
    if [ "$keep_log" -eq 0 ] && [ -f "$log" ]; then
        rm -f "$log"
    fi
}
trap cleanup EXIT

set +e
timeout --foreground "${timeout_s}s" \
    make -C "$repo_root" ARCH=x86_64 BOARD=qemu-asterinas run >"$log" 2>&1
status=$?
set -e

if [ "$status" -ne 0 ] && [ "$status" -ne 124 ]; then
    tail -n 120 "$log" >&2
    exit "$status"
fi

require_log() {
    local pattern="$1"
    local label="$2"
    if ! grep -Eq "$pattern" "$log"; then
        echo "error: missing $label in qemu-asterinas serial output" >&2
        tail -n 120 "$log" >&2
        exit 1
    fi
}

require_log "Entering the Asterinas entry point" "Asterinas entry marker"
require_log "asterinas-on-hvisor-ok" "userspace marker"
require_log "BusyBox|init complete" "shell/init marker"

echo "PASS: qemu-asterinas reached Asterinas userspace"
