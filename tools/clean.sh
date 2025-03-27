#!/bin/bash

set -e

function find_hvisor_src {
    # find in ./CHANGELOG.md ../CHANGELOG.md
    ret=$(find . -name CHANGELOG.md -exec grep -l "hvisor" {} \;)
    if [ -z "$ret" ]; then
        ret=$(find .. -name CHANGELOG.md -exec grep -l "hvisor" {} \;)
    fi
    if [ -z "$ret" ]; then
        echo "."
    fi
    echo $(dirname $ret)
}

HVISOR_SRC=$(realpath $(find_hvisor_src))

cd $HVISOR_SRC

cargo clean
rm -f .config
rm -f .cargo/config.toml
rm -f src/platform/__board.rs

echo "cleaned hvisor source tree at $(date)"
