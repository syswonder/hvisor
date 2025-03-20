#!/bin/bash

set -e

function find_hvisor_src {
    # get the directory which contains CHANGELOG.md and "hvisor" in the file contents
    # this is the root of the hvisor source tree, return the absolute path
    ret=$(find . -name CHANGELOG.md -exec grep -q hvisor {} \; -print -quit)
    if [ -z "$ret" ]; then
        echo "Could not find hvisor source tree"
        exit 1
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