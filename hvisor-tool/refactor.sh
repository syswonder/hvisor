#!/bin/bash

find ./tools/ ./include/ ./driver/ -name "*.c" ! -name "*.mod.c" -o -name "*.h" | xargs clang-format-14 -i