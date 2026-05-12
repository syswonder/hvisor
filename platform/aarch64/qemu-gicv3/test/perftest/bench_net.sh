#!/bin/sh
# Zone0 virtio-net benchmark: RTT ping + iperf3 loopback throughput

RESULT_DIR="./test/perfresult"
mkdir -p "$RESULT_DIR"
OUTFILE="$RESULT_DIR/bench_net.txt"
GATEWAY="10.0.2.2"

echo "=== Network Benchmark (zone0) ===" | tee "$OUTFILE"
echo "Date: $(date)" | tee -a "$OUTFILE"
echo "" | tee -a "$OUTFILE"

echo "[/proc/net/dev]" | tee -a "$OUTFILE"
if [ -r /proc/net/dev ]; then
    cat /proc/net/dev | tee -a "$OUTFILE"
else
    echo "/proc/net/dev unavailable" | tee -a "$OUTFILE"
fi
echo "" | tee -a "$OUTFILE"

echo "[ping ${GATEWAY} -c 50 -i 0.02]" | tee -a "$OUTFILE"
PING_LOG=$(mktemp /tmp/hv_ping.XXXXXX)
if ping -c 50 -i 0.02 "$GATEWAY" >"$PING_LOG" 2>&1; then
    cat "$PING_LOG" | tee -a "$OUTFILE"
    echo "ping completed" | tee -a "$OUTFILE"
else
    cat "$PING_LOG" | tee -a "$OUTFILE"
    echo "ping failed or network not available" | tee -a "$OUTFILE"
fi
rm -f "$PING_LOG"
echo "" | tee -a "$OUTFILE"

echo "[loopback readiness]" | tee -a "$OUTFILE"
if command -v ip > /dev/null 2>&1; then
    ip link set lo up >/dev/null 2>&1 || true
    echo "attempted: ip link set lo up" | tee -a "$OUTFILE"
elif command -v ifconfig > /dev/null 2>&1; then
    ifconfig lo up >/dev/null 2>&1 || true
    echo "attempted: ifconfig lo up" | tee -a "$OUTFILE"
else
    echo "no ip/ifconfig command, cannot explicitly bring up lo" | tee -a "$OUTFILE"
fi

LO_LOG=$(mktemp /tmp/hv_lo_ping.XXXXXX)
if ping -c 1 127.0.0.1 >"$LO_LOG" 2>&1; then
    cat "$LO_LOG" | tee -a "$OUTFILE"
    LO_READY=1
else
    cat "$LO_LOG" | tee -a "$OUTFILE"
    LO_READY=0
fi
rm -f "$LO_LOG"
echo "" | tee -a "$OUTFILE"

if command -v iperf3 > /dev/null 2>&1; then
    if [ "$LO_READY" -eq 1 ]; then
        echo "[iperf3 loopback: server on 127.0.0.1:5201, duration 10s]" | tee -a "$OUTFILE"
        IPERF_SRV_LOG=$(mktemp /tmp/hv_iperf3_srv.XXXXXX)
        IPERF_CLI_LOG=$(mktemp /tmp/hv_iperf3_cli.XXXXXX)
        iperf3 -s -p 5201 >"$IPERF_SRV_LOG" 2>&1 &
        IPERF_PID=$!
        sleep 1
        if ! kill -0 "$IPERF_PID" 2>/dev/null; then
            echo "iperf3 server start failed" | tee -a "$OUTFILE"
            cat "$IPERF_SRV_LOG" | tee -a "$OUTFILE"
        else
            if iperf3 -c 127.0.0.1 -p 5201 -t 10 >"$IPERF_CLI_LOG" 2>&1; then
                cat "$IPERF_CLI_LOG" | tee -a "$OUTFILE"
                echo "iperf3 loopback completed" | tee -a "$OUTFILE"
            else
                cat "$IPERF_CLI_LOG" | tee -a "$OUTFILE"
                echo "iperf3 client failed" | tee -a "$OUTFILE"
            fi
        fi
        kill "$IPERF_PID" 2>/dev/null || true
        wait "$IPERF_PID" 2>/dev/null || true
        rm -f "$IPERF_SRV_LOG" "$IPERF_CLI_LOG"
    else
        echo "loopback not ready, skipping iperf3 loopback test" | tee -a "$OUTFILE"
    fi
else
    echo "[iperf3 not found — skipping throughput test]" | tee -a "$OUTFILE"
fi
echo "" | tee -a "$OUTFILE"

echo "=== Done ===" | tee -a "$OUTFILE"
