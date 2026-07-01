#!/usr/bin/env bash
# Write CONFIG_* lines from repo-root .config to the board defconfig (sorted).
set -euo pipefail
ARCH="${1:?arch}"
BOARD="${2:?board}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

OUT="platform/${ARCH}/${BOARD}/kconfig/defconfig"
mkdir -p "$(dirname "$OUT")"
if [[ ! -f .config ]]; then
	echo "error: no .config in repo root" >&2
	exit 1
fi
grep '^CONFIG_' .config | LC_ALL=C sort >"$OUT"
echo "wrote $OUT"
