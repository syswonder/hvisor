#!/bin/sh
# Non-interactive serial check for inner zone virtio-console (/dev/pts/x).
# Usage: check_serial.sh <pts_dev> <log_file> <command...>

set -eu

pts_dev=${1:?pts device required}
log_file=${2:?log file required}
shift 2

: > "$log_file"

stty -F "$pts_dev" 115200 cs8 -cstopb -parenb -ixon -ixoff -echo 2>/dev/null \
    || stty -F "$pts_dev" raw -echo 2>/dev/null \
    || true

{
    printf '%s\r\n' "$*"
    sleep 3
    timeout 10 cat "$pts_dev" 2>/dev/null || true
} >> "$log_file" 2>&1

if [ ! -s "$log_file" ]; then
    echo "check_serial: no output from $pts_dev" >> "$log_file"
    exit 1
fi

exit 0
