#!/bin/sh
# Zone1 virtio-blk benchmark: fio preferred, dd fallback

RESULT_DIR="./test/perfresult"
mkdir -p "$RESULT_DIR"
OUTFILE="$RESULT_DIR/bench_blk.txt"

echo "=== Virtio-BLK Benchmark (Zone1) ===" | tee "$OUTFILE"
echo "Date: $(date)" | tee -a "$OUTFILE"
echo "" | tee -a "$OUTFILE"

ROOT_DEV="$(findmnt -n -o SOURCE / 2>/dev/null || true)"
if [ -n "$ROOT_DEV" ]; then
    echo "[root device] $ROOT_DEV" | tee -a "$OUTFILE"
fi

echo "[writeback sync]" | tee -a "$OUTFILE"
sync

echo "" | tee -a "$OUTFILE"
if command -v fio > /dev/null 2>&1; then
    FIO_FILE="/home/arm64/test/perfresult/fio_blk_bench.dat"
    echo "[fio seq-write 128M -> ${FIO_FILE}]" | tee -a "$OUTFILE"
    fio --name=blk-seq-write --rw=write --bs=1M --size=128M --numjobs=1 \
        --filename="${FIO_FILE}" --direct=1 --ioengine=libaio --iodepth=16 \
        --end_fsync=1 --output-format=normal 2>&1 | tee -a "$OUTFILE"
    echo "" | tee -a "$OUTFILE"

    echo "[fio seq-read 128M <- ${FIO_FILE}]" | tee -a "$OUTFILE"
    fio --name=blk-seq-read --rw=read --bs=1M --size=128M --numjobs=1 \
        --filename="${FIO_FILE}" --direct=1 --ioengine=libaio --iodepth=16 \
        --output-format=normal 2>&1 | tee -a "$OUTFILE"
    rm -f "${FIO_FILE}"
else
    BENCH_FILE="/home/arm64/test/perfresult/hv_blk_bench.dd"
    echo "[fio not found - fallback to dd]" | tee -a "$OUTFILE"
    echo "[Write: dd if=/dev/zero of=${BENCH_FILE} bs=1M count=128 conv=fdatasync]" | tee -a "$OUTFILE"
    dd if=/dev/zero of="${BENCH_FILE}" bs=1M count=128 conv=fdatasync 2>&1 | tee -a "$OUTFILE"
    echo "" | tee -a "$OUTFILE"

    echo "[Read: dd if=${BENCH_FILE} of=/dev/null bs=1M]" | tee -a "$OUTFILE"
    dd if="${BENCH_FILE}" of=/dev/null bs=1M 2>&1 | tee -a "$OUTFILE"
    rm -f "${BENCH_FILE}"
fi

echo "" | tee -a "$OUTFILE"
echo "=== Done ===" | tee -a "$OUTFILE"
