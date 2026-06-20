#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

seconds="${1:-300}"
net_target="${2:-10.0.0.1}"
base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bin_dir="${base_dir}/../build/bin"

pids=()
cleanup() {
  for pid in "${pids[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT INT TERM

"${base_dir}/cpu_burn.sh" 2 "$seconds" &
pids+=("$!")

if [ -x "${bin_dir}/mem_burn" ]; then
  "${bin_dir}/mem_burn" --chunk-mb 64 --seconds "$seconds" &
  pids+=("$!")
fi

"${base_dir}/io_burn.sh" "$seconds" &
pids+=("$!")

"${base_dir}/net_burn.sh" "$net_target" "$seconds" &
pids+=("$!")

wait
