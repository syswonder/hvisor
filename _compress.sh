#!/bin/bash

INPUT_FILE="flash.img"
if [ ! -f "$INPUT_FILE" ]; then
  echo "Error: File $INPUT_FILE does not exist."
  exit 1
fi

OUTPUT_FILE="flash.img.partial"

dd if="$INPUT_FILE" of="$OUTPUT_FILE" bs=1K count=1 status=none

if [ $? -eq 0 ]; then
  echo "Successfully truncated $INPUT_FILE to $OUTPUT_FILE."
else
  echo "Error: Truncation failed."
  exit 1
fi