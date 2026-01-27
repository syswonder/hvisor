#!/bin/bash
set -e

function find_hvisor_src {
    ret=$(find . -name CHANGELOG.md -exec grep -l "hvisor" {} \;)
    if [ -z "$ret" ]; then
        ret=$(find .. -name CHANGELOG.md -exec grep -l "hvisor" {} \;)
    fi
    if [ -z "$ret" ]; then
        echo "ERROR: Could not locate hvisor source root (CHANGELOG.md with 'hvisor')" >&2
        exit 1
    fi
    echo "$(dirname "$ret")"
}

HVISOR_SRC=$(realpath $(find_hvisor_src))
cd "$HVISOR_SRC"

if [ -n "$BID" ]; then
    ARCH=$(echo "$BID" | cut -d'/' -f1)
    BOARD=$(echo "$BID" | cut -d'/' -f2)
fi

if [ -z "$ARCH" ] || [ -z "$BOARD" ]; then
    echo "ERROR: ARCH/BOARD not set, please use BID=arch/board or ARCH=... BOARD=..."
    exit 1
fi

if [ -z "$FEATURES" ]; then
    FEATURES=$(./tools/read_features.sh "$ARCH" "$BOARD")
fi

FEATURES=$(echo "$FEATURES" | tr '\n' ' ' | tr -s ' ' | sed 's/^ *//;s/ *$//')

case "$ARCH" in
    aarch64) TARGET="aarch64-unknown-none" ;;
    riscv64) TARGET="riscv64gc-unknown-none-elf" ;;
    loongarch64) TARGET="loongarch64-unknown-none" ;;
    x86_64) TARGET="x86_64-unknown-none" ;;
    *)
        echo "ERROR: Unsupported ARCH value: $ARCH"
        exit 1
        ;;
esac

mkdir -p .vscode

cat > .vscode/settings.json <<EOF
{
	"rust-analyzer.linkedProjects": [
		"./Cargo.toml"
	],
	"rust.target": "$TARGET",
	"rust.all_targets": false,
	"rust-analyzer.cargo.target": "$TARGET",
	"rust-analyzer.cargo.features": [
		"$FEATURES"
	]
}
EOF

echo "generated .vscode/settings.json for ARCH=$ARCH BOARD=$BOARD TARGET=$TARGET"