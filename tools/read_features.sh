#!/bin/bash

set -e

ARCH=$1
BOARD=$2

FEATURES="platform/$ARCH/$BOARD/cargo/features"
# read the contents of the features file
FEATURES=$(cat $FEATURES)
# post process the features, replace any newlines with spaces
FEATURES=$(echo $FEATURES | tr '\n' ' ')
FEATURES=$(echo $FEATURES | tr -d '\r')
# print the features
echo $FEATURES
