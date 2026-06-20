#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

seconds="${1:-300}"
target="${2:-/tmp/hvisor_io_burn.bin}"
end=$((SECONDS + seconds))

while [ "$SECONDS" -lt "$end" ]; do
  dd if=/dev/zero of="$target" bs=1M count=256 conv=fdatasync status=none
  rm -f "$target"
done
