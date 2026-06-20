#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

workers="${1:-2}"
seconds="${2:-300}"

if command -v stress >/dev/null 2>&1; then
  exec stress --cpu "$workers" --timeout "$seconds"
fi

pids=()
cleanup() {
  for pid in "${pids[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT INT TERM

for _ in $(seq 1 "$workers"); do
  sh -c 'while :; do :; done' &
  pids+=("$!")
done
sleep "$seconds"
