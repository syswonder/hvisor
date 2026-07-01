#!/bin/sh
# Relay power control for board CI (socat hex frames).
# Usage: board_power.sh off|on|cycle <port>
#   cycle: off, wait 3s, on

set -eu

action=${1:?action required (off|on|cycle)}
port=${2:?serial port required, e.g. /dev/ttyUSB1}

send_frame() {
    frame=$1
    printf '%b' "$frame" | sudo socat - "$port"
}

case "$action" in
    off)
        send_frame '\xA0\x04\x00\xA4'
        ;;
    on)
        send_frame '\xA0\x04\x01\xA5'
        ;;
    cycle)
        send_frame '\xA0\x04\x00\xA4'
        sleep 3
        send_frame '\xA0\x04\x01\xA5'
        ;;
    *)
        echo "usage: $0 off|on|cycle <port>" >&2
        exit 1
        ;;
esac
