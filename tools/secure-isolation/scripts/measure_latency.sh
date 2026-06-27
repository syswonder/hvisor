#!/usr/bin/env bash
set -euo pipefail

ITERS="${1:-10000}"
OUT="${2:-latency.log}"
: > "$OUT"
PROBE_FILE="$(mktemp "${TMPDIR:-.}/hvisor_latency_probe.XXXXXX")"
trap 'rm -f "$PROBE_FILE"' EXIT INT TERM

i=0
while [ "$i" -lt "$ITERS" ]; do
    start="$(date +%s%N)"
    printf x > "$PROBE_FILE"
    cat "$PROBE_FILE" >/dev/null
    end="$(date +%s%N)"
    echo $((end - start)) >> "$OUT"
    i=$((i + 1))
done

sort -n "$OUT" | awk '
{ a[NR] = $1 }
END {
    if (NR == 0) { exit 1 }
    p50 = int(NR * 0.50); if (p50 < 1) p50 = 1
    p99 = int(NR * 0.99); if (p99 < 1) p99 = 1
    p999 = int(NR * 0.999); if (p999 < 1) p999 = 1
    print "samples=" NR
    print "p50_ns=" a[p50]
    print "p99_ns=" a[p99]
    print "p999_ns=" a[p999]
}'
