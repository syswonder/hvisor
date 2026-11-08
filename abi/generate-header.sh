#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

out="$1"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

cbindgen --config abi/cbindgen.toml --crate hvisor-abi --output "$tmp"

python3 - "$tmp" <<'PY'
import sys
from pathlib import Path

p = Path(sys.argv[1])
s = p.read_text()
old = """#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
"""
new = """#if defined(__KERNEL__)
#include <linux/types.h>
#else
#include <stdbool.h>
#include <stdint.h>
#endif
"""
if old not in s:
    raise SystemExit("unexpected cbindgen system includes")
p.write_text(s.replace(old, new, 1))
PY

cp "$tmp" "$out"
