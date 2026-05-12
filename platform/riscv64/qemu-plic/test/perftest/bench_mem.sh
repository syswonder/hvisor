#!/bin/sh
# Zone0 memory benchmark: stress-ng + fio when available; dd fallback
RESULT_DIR="./test/perfresult"
mkdir -p "$RESULT_DIR"
OUTFILE="$RESULT_DIR/bench_mem.txt"
echo "=== Memory Benchmark ===" | tee "$OUTFILE"
echo "Date: $(date)" | tee -a "$OUTFILE"
echo "" | tee -a "$OUTFILE"
if command -v stress-ng > /dev/null 2>&1; then
    echo "[stress-ng --vm 1 --vm-bytes 64M --vm-method all --timeout 20s --metrics-brief]" | tee -a "$OUTFILE"
    stress-ng --vm 1 --vm-bytes 64M --vm-method all --timeout 20s --metrics-brief 2>&1 | tee -a "$OUTFILE"
    echo "" | tee -a "$OUTFILE"
else
    echo "[stress-ng not found — skipping memory stress]" | tee -a "$OUTFILE"
    echo "" | tee -a "$OUTFILE"
fi
if command -v fio > /dev/null 2>&1; then
    BENCH_DIR=/tmp
    FIO_FILE="$BENCH_DIR/fio_mem_bench"
    AVAIL_KIB=$(df -Pk "$BENCH_DIR" 2>/dev/null | awk 'NR==2{print $4}')
    FIO_SIZE_MIB=128
    if [ -n "$AVAIL_KIB" ] && [ "$AVAIL_KIB" -gt 0 ] 2>/dev/null; then
        AVAIL_MIB=$((AVAIL_KIB / 1024))
        if [ "$AVAIL_MIB" -gt 16 ]; then
            FIO_SIZE_MIB=$((AVAIL_MIB * 7 / 10))
            if [ "$FIO_SIZE_MIB" -gt 128 ]; then
                FIO_SIZE_MIB=128
            fi
            if [ "$FIO_SIZE_MIB" -lt 16 ]; then
                FIO_SIZE_MIB=16
            fi
        elif [ "$AVAIL_MIB" -ge 10 ]; then
            FIO_SIZE_MIB=8
        else
            FIO_SIZE_MIB=0
        fi
    fi

    rm -f "${FIO_FILE}"
    if [ "$FIO_SIZE_MIB" -eq 0 ]; then
        echo "[fio skipped: insufficient free space on $BENCH_DIR]" | tee -a "$OUTFILE"
        echo "" | tee -a "$OUTFILE"
    else
        echo "[fio seq-write ${FIO_SIZE_MIB}M -> $BENCH_DIR]" | tee -a "$OUTFILE"
        if fio --name=seq-write --rw=write --bs=1M --size="${FIO_SIZE_MIB}M" --numjobs=1 \
            --filename="${FIO_FILE}" --end_fsync=1 --output-format=normal 2>&1 | tee -a "$OUTFILE"; then
            echo "" | tee -a "$OUTFILE"
            echo "[fio seq-read ${FIO_SIZE_MIB}M <- $BENCH_DIR]" | tee -a "$OUTFILE"
            fio --name=seq-read --rw=read --bs=1M --size="${FIO_SIZE_MIB}M" --numjobs=1 \
                --filename="${FIO_FILE}" --output-format=normal 2>&1 | tee -a "$OUTFILE"
        else
            echo "" | tee -a "$OUTFILE"
            echo "[fio write failed — skipping seq-read]" | tee -a "$OUTFILE"
        fi
        rm -f "${FIO_FILE}"
        echo "" | tee -a "$OUTFILE"
    fi
else
    echo "[fio not found — falling back to dd]" | tee -a "$OUTFILE"
    BENCH_DIR=/tmp
    BENCH_FILE="$BENCH_DIR/hv_bench_mem"
    AVAIL_KIB=$(df -Pk "$BENCH_DIR" 2>/dev/null | awk 'NR==2{print $4}')
    DD_COUNT_MIB=64
    if [ -n "$AVAIL_KIB" ] && [ "$AVAIL_KIB" -gt 0 ] 2>/dev/null; then
        AVAIL_MIB=$((AVAIL_KIB / 1024))
        if [ "$AVAIL_MIB" -gt 16 ]; then
            DD_COUNT_MIB=$((AVAIL_MIB * 7 / 10))
            if [ "$DD_COUNT_MIB" -gt 64 ]; then
                DD_COUNT_MIB=64
            fi
            if [ "$DD_COUNT_MIB" -lt 8 ]; then
                DD_COUNT_MIB=8
            fi
        elif [ "$AVAIL_MIB" -ge 10 ]; then
            DD_COUNT_MIB=8
        else
            DD_COUNT_MIB=0
        fi
    fi

    rm -f "$BENCH_FILE"
    if [ "$DD_COUNT_MIB" -eq 0 ]; then
        echo "[dd skipped: insufficient free space on $BENCH_DIR]" | tee -a "$OUTFILE"
        echo "" | tee -a "$OUTFILE"
    else
        echo "[Write: dd if=/dev/zero of=$BENCH_FILE bs=1M count=$DD_COUNT_MIB conv=fdatasync]" | tee -a "$OUTFILE"
        if dd if=/dev/zero of="$BENCH_FILE" bs=1M count="$DD_COUNT_MIB" conv=fdatasync 2>&1 | tee -a "$OUTFILE"; then
            echo "" | tee -a "$OUTFILE"
            echo "[Read: dd if=$BENCH_FILE of=/dev/null bs=1M]" | tee -a "$OUTFILE"
            dd if="$BENCH_FILE" of=/dev/null bs=1M 2>&1 | tee -a "$OUTFILE"
        else
            echo "" | tee -a "$OUTFILE"
            echo "[dd write failed — skipping read]" | tee -a "$OUTFILE"
        fi
        echo "" | tee -a "$OUTFILE"
        rm -f "$BENCH_FILE"
    fi
fi

echo "" | tee -a "$OUTFILE"
echo "=== Done ===" | tee -a "$OUTFILE"
