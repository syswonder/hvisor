#!/bin/sh
# Non-interactive virtio-console check for inner zone (/dev/pts/x).
# Usage: check_serial.sh <pts_dev> <log_file> [timeout_sec] [command...]

set -eu

pts_dev=${1:?pts device required}
log_file=${2:?log file required}
shift 2

prompt_timeout=60
if [ $# -gt 0 ] && [ "$1" -eq "$1" ] 2>/dev/null; then
    prompt_timeout=$1
    shift
fi

: > "$log_file"

read_pts() {
    timeout "${1:-2}" cat "$pts_dev" 2>/dev/null >> "$log_file" || true
}

has_console_ready() {
    tr -d '\r' < "$log_file" | sed 's/\x1b\[[0-9;?]*[ -\/]*[@-~]//g' \
        | grep -qE 'root@[^[:space:]]*[#$][[:space:]]*$|^[[:space:]]*#[[:space:]]*$|login:[[:space:]]*$'
}

deadline=$(( $(date +%s) + prompt_timeout ))
while [ "$(date +%s)" -lt "$deadline" ]; do
    printf '\r\n' > "$pts_dev" 2>/dev/null || true
    read_pts 2
    if has_console_ready; then
        break
    fi
    sleep 0.2
done

if ! has_console_ready; then
    echo "check_serial: timed out after ${prompt_timeout}s waiting for console prompt on $pts_dev" \
        >> "$log_file"
    exit 1
fi

if [ $# -gt 0 ]; then
    printf '%s\r\n' "$@"
    sleep 1
    read_pts 10
fi

exit 0
