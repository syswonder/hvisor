#!/bin/sh
# Relay power control for board CI (socat hex frames).
# Usage: board_power.sh off|on|cycle <port> [channel]
#   cycle: off, wait 3s, on

set -eu

action=${1:?action required (off|on|cycle)}
port=${2:?serial port required, e.g. /dev/ttyUSB1}
channel=${3:-4}
lock_file=${RELAY_LOCK_FILE:-/var/lock/hvisor-relay.lock}
lock_timeout=${RELAY_LOCK_TIMEOUT:-300}

if ! command -v flock >/dev/null 2>&1; then
    echo "error: flock is required for relay serial locking" >&2
    exit 1
fi

case "${channel}" in
    1|2|3|4) ;;
    *)
        echo "invalid relay channel: ${channel}" >&2
        exit 1
        ;;
esac

checksum() {
    state=$1
    printf '%02X' "$((0xA0 + channel + state))"
}

send_frame() {
    state=$1
    frame=$(printf '%s%02X%s%02X%s%s' '\\xA0\\x' "${channel}" '\\x' "${state}" '\\x' "$(checksum "${state}")")
    printf '%b' "$frame" | sudo socat - "$port,b9600,raw,echo=0"
}

run_locked() {
    if ! flock -w "${lock_timeout}" 9; then
        echo "error: timed out waiting for relay lock: ${lock_file}" >&2
        exit 1
    fi

    case "$action" in
        off)
            send_frame 0
            ;;
        on)
            send_frame 1
            ;;
        cycle)
            send_frame 0
            sleep 3
            send_frame 1
            ;;
        *)
            echo "usage: $0 off|on|cycle <port> [channel]" >&2
            exit 1
            ;;
    esac
}

exec 9>"${lock_file}"

case "$action" in
    off|on|cycle)
        run_locked
        ;;
    *)
        echo "usage: $0 off|on|cycle <port> [channel]" >&2
        exit 1
        ;;
esac
