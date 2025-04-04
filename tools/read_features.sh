#!/bin/bash

set -e

ARCH=$1
BOARD=$2

FEATURES="platform/$ARCH/$BOARD/cargo/features"
FEATURES=$(cat $FEATURES)
FEATURES=$(echo $FEATURES | tr '\n' ' ')
FEATURES=$(echo $FEATURES | tr -d '\r')
echo $FEATURES
