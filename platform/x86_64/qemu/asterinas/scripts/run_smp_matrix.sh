#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HVISOR_BIN="${HVISOR_BIN:-./hvisor}"
CONFIG_DIR="${CONFIG_DIR:-$ROOT/configs}"
RESULTS="${RESULTS:-$ROOT/results/$(date +%Y%m%d_%H%M%S)}"
ASTERINAS_RUNNER="${ASTERINAS_RUNNER:-}"

mkdir -p "$RESULTS"

if [ -z "$ASTERINAS_RUNNER" ]; then
  cat >&2 <<'EOF'
ASTERINAS_RUNNER is required.

It must be a command available from zone0 that runs a command inside the
Asterinas zone and copies stdout back to zone0. Examples are a serial-console
expect script, ssh wrapper, or hvisor virtio-console helper.

Expected interface:
  $ASTERINAS_RUNNER ./smp_probe ...
  $ASTERINAS_RUNNER './timer_jitter --period-us 1000 --duration 60'
EOF
  exit 2
fi

copy_config() {
  local name="$1"
  printf '%s/%s' "$CONFIG_DIR" "$name"
}

run_probe_set() {
  local tag="$1"
  local ncpu="$2"

  "$ASTERINAS_RUNNER" "/bin/smp_probe --threads $ncpu --iterations 10000000" \
    > "$RESULTS/smp_${tag}.txt"

  if [ "$ncpu" -ge 2 ]; then
    "$ASTERINAS_RUNNER" "/bin/ipi_pingpong --iterations 100000 --cpu-a 0 --cpu-b 1" \
      > "$RESULTS/ipi_${tag}.txt"
    python3 "$ROOT/pipeline/stats.py" "$RESULTS/ipi_${tag}.txt" \
      > "$RESULTS/ipi_${tag}_stats.txt"
  fi

  "$ASTERINAS_RUNNER" "/bin/timer_jitter --period-us 1000 --duration 60" \
    > "$RESULTS/jitter_${tag}.txt"
  python3 "$ROOT/pipeline/stats.py" --unit ns "$RESULTS/jitter_${tag}.txt" \
    > "$RESULTS/jitter_${tag}_stats.txt"

  "$ASTERINAS_RUNNER" "/bin/ctxsw --iterations 100000" \
    > "$RESULTS/ctxsw_${tag}.txt"
}

python3 "$ROOT/scripts/check_configs.py" --configs-dir "$CONFIG_DIR" --matrix

declare -A noisy_cfg_for
noisy_cfg_for[1]="zone2_noisy.json"
noisy_cfg_for[2]="zone2_noisy.json"
noisy_cfg_for[4]="zone2_noisy_8cpu.json"

for ncpu in 1 2 4; do
  zone_cfg="zone1_aster_${ncpu}c.json"
  for mode in quiet noisy; do
    tag="${ncpu}c_${mode}"
    echo "==> $tag"
    "$HVISOR_BIN" zone start "$(copy_config "$zone_cfg")"
    sleep 5

    if [ "$mode" = noisy ]; then
      "$HVISOR_BIN" zone start "$(copy_config "${noisy_cfg_for[$ncpu]}")"
      sleep 3
    fi

    run_probe_set "$tag" "$ncpu"

    if [ "$mode" = noisy ]; then
      "$HVISOR_BIN" zone shutdown 2 || true
    fi
    "$HVISOR_BIN" zone shutdown 1 || true
    sleep 2
  done
done

echo "results: $RESULTS"
