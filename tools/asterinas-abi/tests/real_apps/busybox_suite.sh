#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -u
if command -v busybox >/dev/null 2>&1; then
    BB=busybox
elif [ -x /bin/busybox ]; then
    BB=/bin/busybox
else
    echo "busybox_suite: SKIP: busybox not installed"
    exit 77
fi

# Required applets: if any is not built into this busybox, the case is not
# applicable (SKIP) rather than a real failure. busybox --list is the canonical
# enumeration of compiled-in applets; if it is itself unavailable we cannot
# determine applicability and skip conservatively.
applet_list="$("$BB" --list 2>/dev/null)"
if [ -z "$applet_list" ]; then
    echo "busybox_suite: SKIP: busybox --list unavailable"
    exit 77
fi
for applet in echo cat grep sort cp chmod ls find date id sed xargs stat; do
    if ! printf '%s\n' "$applet_list" | grep -qx "$applet"; then
        echo "busybox_suite: SKIP: applet '$applet' not available"
        exit 77
    fi
done

tmp=/tmp/abi_busybox_suite
rm -rf "$tmp"
mkdir -p "$tmp/sub"
printf 'c\nb\na\n' > "$tmp/input"

fail() { echo "busybox_suite: FAIL: $1"; exit 1; }

# Run each applet and check its exit status individually.
"$BB" echo hello >/tmp/abi_bb_echo            || fail "echo rc=$?"
"$BB" cat "$tmp/input" >/tmp/abi_bb_cat       || fail "cat rc=$?"
"$BB" grep b "$tmp/input" >/tmp/abi_bb_grep   || fail "grep rc=$?"
"$BB" sort "$tmp/input" >/tmp/abi_bb_sort     || fail "sort rc=$?"
"$BB" cp "$tmp/input" "$tmp/copy"             || fail "cp rc=$?"
"$BB" chmod 600 "$tmp/copy"                   || fail "chmod rc=$?"
"$BB" ls "$tmp" >/tmp/abi_bb_ls               || fail "ls rc=$?"
"$BB" find "$tmp" -type f >/tmp/abi_bb_find   || fail "find rc=$?"
"$BB" date >/tmp/abi_bb_date                  || fail "date rc=$?"
"$BB" id >/tmp/abi_bb_id                      || fail "id rc=$?"
"$BB" sed 's/a/A/' "$tmp/input" >/tmp/abi_bb_sed || fail "sed rc=$?"
"$BB" xargs echo </tmp/abi_bb_sort >/tmp/abi_bb_xargs || fail "xargs rc=$?"

for f in /tmp/abi_bb_echo /tmp/abi_bb_cat /tmp/abi_bb_grep /tmp/abi_bb_sort \
         /tmp/abi_bb_ls /tmp/abi_bb_find /tmp/abi_bb_date /tmp/abi_bb_id \
         /tmp/abi_bb_sed /tmp/abi_bb_xargs "$tmp/copy"; do
    [ -s "$f" ] || fail "empty $f"
done

# cp + chmod must have produced a 0600 copy of the input.
mode="$("$BB" stat -c '%a' "$tmp/copy" 2>/dev/null)"
[ "$mode" = "600" ] || fail "copy mode=$mode (expected 600)"
if command -v cmp >/dev/null 2>&1; then
    cmp -s "$tmp/input" "$tmp/copy" || fail "copy content mismatch"
elif printf '%s\n' "$applet_list" | grep -qx cmp; then
    "$BB" cmp -s "$tmp/input" "$tmp/copy" || fail "copy content mismatch"
fi

rm -rf "$tmp"
echo "busybox_suite: PASS"
