#!/bin/sh
# Zone0 IRQ statistics and timer latency measurement
# Uses cyclictest (rt-tests) when available; falls back to date-based sleep jitter

RESULT_DIR="./test/perfresult"
mkdir -p "$RESULT_DIR"
OUTFILE="$RESULT_DIR/bench_irq.txt"

echo "=== IRQ / Timer Latency Benchmark ===" | tee "$OUTFILE"
echo "Date: $(date)" | tee -a "$OUTFILE"
echo "" | tee -a "$OUTFILE"

echo "[/proc/interrupts]" | tee -a "$OUTFILE"
if [ -r /proc/interrupts ]; then
    cat /proc/interrupts | tee -a "$OUTFILE"
else
    echo "/proc/interrupts unavailable" | tee -a "$OUTFILE"
fi
echo "" | tee -a "$OUTFILE"

run_sleep_jitter() {
    i=0
    while [ $i -lt 20 ]; do
        t1=$(date +%s%N 2>/dev/null)
        sleep 0.01
        t2=$(date +%s%N 2>/dev/null)
        if echo "${t1}${t2}" | grep -qE '^[0-9]+$'; then
            delta=$((t2 - t1))
            echo "  sample $((i+1)): ${delta} ns  (expected ~10000000)" | tee -a "$OUTFILE"
        else
            echo "  sample $((i+1)): nanosecond timestamp unavailable" | tee -a "$OUTFILE"
        fi
        i=$((i+1))
    done
}

if command -v cyclictest > /dev/null 2>&1; then
    echo "[cyclictest -l 1000 -i 1000 -t 1 (interval=1ms, 1000 loops)]" | tee -a "$OUTFILE"
    TMP_OUT=$(mktemp /tmp/hv_cyclictest.XXXXXX)
    if cyclictest -l 1000 -i 1000 -t 1 >"$TMP_OUT" 2>&1; then
        cat "$TMP_OUT" | tee -a "$OUTFILE"
    else
        cat "$TMP_OUT" | tee -a "$OUTFILE"
        echo "[cyclictest failed — falling back to 10ms sleep jitter, 20 samples]" | tee -a "$OUTFILE"
        run_sleep_jitter
    fi
    rm -f "$TMP_OUT"
else
    echo "[cyclictest not found — falling back to 10ms sleep jitter, 20 samples]" | tee -a "$OUTFILE"
    run_sleep_jitter
fi
echo "" | tee -a "$OUTFILE"

echo "=== Done ===" | tee -a "$OUTFILE"
