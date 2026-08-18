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

case "${channel}" in
    1|2|3|4) ;;
    *)
        echo "invalid relay channel: ${channel}" >&2
        exit 1
        ;;
esac

socat_serial_addr() {
    # by-path device names contain ':'; socat treats them as option separators.
    case "$port" in
        *:*)
            escaped=$(printf '%s' "$port" | sed 's/:/\\:/g')
            printf 'GOPEN:%s,b9600,raw,echo=0' "$escaped"
            ;;
        *)
            printf '%s,b9600,raw,echo=0' "$port"
            ;;
    esac
}

send_frame() {
    state=$1
    checksum=$((0xA0 + channel + state))
    printf '[power] send port=%s channel=%s state=%s frame=A0%02X%02X%02X\n' \
        "$port" "$channel" "$state" "$channel" "$state" "$checksum"
    frame=$(printf '\\%03o\\%03o\\%03o\\%03o' 160 "$channel" "$state" "$checksum")
    addr=$(socat_serial_addr)
    if [ "$(id -u)" -eq 0 ]; then
        printf '%b' "$frame" | socat - "$addr" >/dev/null
    else
        printf '%b' "$frame" | sudo socat - "$addr" >/dev/null
    fi
}

command -v flock >/dev/null 2>&1 || {
    echo "error: flock is required for relay serial locking" >&2
    exit 1
}

exec 9>"${lock_file}"
flock -w "${lock_timeout}" 9 || {
    echo "error: timed out waiting for relay lock: ${lock_file}" >&2
    exit 1
}

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
