#!/bin/bash

# first find hvisor src dir by searching for README.md
# find in current dir and parent dir
HVISOR_SRC_DIR=.
while [ ! -f "$HVISOR_SRC_DIR/README.md" ]; do
  if [ "$HVISOR_SRC_DIR" = "/" ]; then
    echo "Error: Could not find hvisor source directory."
    exit 1
  fi
  HVISOR_SRC_DIR="$HVISOR_SRC_DIR/.."
done

INPUT_FILE="$HVISOR_SRC_DIR/flash.img"

if [ ! -f "$INPUT_FILE" ]; then
  echo "Error: File $INPUT_FILE does not exist."
  exit 1
fi

OUTPUT_FILE="$HVISOR_SRC_DIR/flash.img.partial"

dd if="$INPUT_FILE" of="$OUTPUT_FILE" bs=1K count=1 status=none

if [ $? -eq 0 ]; then
  echo "Successfully truncated $INPUT_FILE to $OUTPUT_FILE."
else
  echo "Error: Truncation failed."
  exit 1
fi