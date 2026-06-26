#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -eu

missing=0

check_file() {
    f="$1"
    if ! head -5 "$f" | grep -q 'SPDX-License-Identifier:'; then
        echo "missing SPDX: $f"
        missing=1
    fi
}

# Files identified by extension/name.
for f in $(find . -type f \
    \( -name '*.c' -o -name '*.h' -o -name '*.py' -o -name '*.sh' -o -name 'Makefile' \) \
    ! -path './_build/*' ! -path './_work/*' ! -path './_toolchain/*' \
    ! -path './_tools/*' \
    ! -path './results/*' ! -path './report/out/*' | sort); do
    check_file "$f"
done

# Extensionless scripts identified by a shell/python shebang (e.g. tool
# wrappers and initramfs/init), so renaming/dropping a path entry can never
# silently exempt a script from the SPDX gate.
for f in $(find . -type f \
    ! -name '*.c' ! -name '*.h' ! -name '*.py' ! -name '*.sh' ! -name 'Makefile' \
    ! -path './_build/*' ! -path './_work/*' ! -path './_toolchain/*' \
    ! -path './_tools/*' ! -path './.git/*' \
    ! -path './results/*' ! -path './report/out/*' | sort); do
    if head -1 "$f" 2>/dev/null | grep -Eq '^#!.*(sh|bash|python)'; then
        check_file "$f"
    fi
done

exit "$missing"
