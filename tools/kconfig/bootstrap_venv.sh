#!/usr/bin/env bash
# Create tools/kconfig/.venv and install kconfiglib (+ PySocks) from vendored wheels
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
VENV="${ROOT}/.venv"
REQ="${ROOT}/requirements.txt"
VEN="${ROOT}/vendor"

if [[ ! -f "${REQ}" ]]; then
	echo "ERROR: missing ${REQ}" >&2
	exit 1
fi
if [[ ! -d "${VEN}" ]] || ! compgen -G "${VEN}/*.whl" > /dev/null; then
	echo "ERROR: missing wheels under ${VEN}/ (run: python3 -m pip download -d ${VEN} -r ${REQ})" >&2
	exit 1
fi

python3 -m venv "${VENV}"
# Do not upgrade pip here: that would hit PyPI and can fail behind socks5:// before PySocks exists.
"${VENV}/bin/pip" install -q --no-index --find-links="${VEN}" -r "${REQ}"
