#!/usr/bin/env bash
# Create tools/kconfig/.venv and install kconfiglib (+ PySocks) from PyPI.
# Local/non-CI only; Jenkins uses a pre-baked venv via symlink
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
VENV="${ROOT}/.venv"
REQ="${ROOT}/requirements.txt"

if [[ ! -f "${REQ}" ]]; then
	echo "ERROR: missing ${REQ}" >&2
	exit 1
fi

if [[ -x "${VENV}/bin/python" ]] && "${VENV}/bin/python" -c "import kconfiglib" 2>/dev/null; then
	exit 0
fi

python3 -m venv "${VENV}"
"${VENV}/bin/pip" install -q -r "${REQ}"
