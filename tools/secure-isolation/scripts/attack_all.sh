#!/usr/bin/env bash
set -euo pipefail

ZONE0_IP="${ZONE0_IP:-}"
DURATION="${DURATION:-300}"

echo "starting CPU pressure"
while :; do :; done &
PIDS="$!"

if command -v stress-ng >/dev/null 2>&1; then
    echo "starting memory pressure"
    stress-ng --vm 2 --vm-bytes 128M --vm-method all --timeout "$DURATION" &
    PIDS="$PIDS $!"
else
    echo "SKIP: stress-ng not found"
fi

echo "starting /dev/zero IO pressure"
dd if=/dev/zero of=/dev/null bs=1M count=100000 &
PIDS="$PIDS $!"

if [ -n "$ZONE0_IP" ] && command -v iperf3 >/dev/null 2>&1; then
    echo "starting iperf3 pressure to $ZONE0_IP"
    iperf3 -c "$ZONE0_IP" -t "$DURATION" -P 8 &
    PIDS="$PIDS $!"
else
    echo "SKIP: set ZONE0_IP and install iperf3 for network pressure"
fi

cleanup() {
    kill $PIDS 2>/dev/null || true
}
trap cleanup EXIT INT TERM

wait
