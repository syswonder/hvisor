#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

target="${1:-10.0.0.1}"
seconds="${2:-300}"

if ping -h 2>&1 | grep -q -- '-w'; then
  exec ping -f -s 1400 -w "$seconds" "$target"
fi

end=$((SECONDS + seconds))
while [ "$SECONDS" -lt "$end" ]; do
  ping -c 1 -s 1400 "$target" >/dev/null || true
done
