#!/bin/sh

INPUT_FILE="flash.img.partial"
if [ ! -f "$INPUT_FILE" ]; then
  echo "Error: File $INPUT_FILE does not exist."
  exit 1
fi

OUTPUT_FILE="flash.img"
TARGET_SIZE=$((64 * 1024)) # 64MB is 64 * 1024KB

CURRENT_SIZE=$(stat -c%s "$INPUT_FILE" 2>/dev/null || stat -f%z "$INPUT_FILE")
CURRENT_SIZE=$((CURRENT_SIZE / 1024)) # Convert bytes to KB

echo "Current size of $INPUT_FILE: $CURRENT_SIZE KB."
echo "Target size: $TARGET_SIZE KB."

if [ $CURRENT_SIZE -gt $TARGET_SIZE ]; then
  echo "Error: $INPUT_FILE is larger than the target size of 64MB."
  exit 1
fi

cp "$INPUT_FILE" "$OUTPUT_FILE"
dd if=/dev/zero bs=1K count=$((TARGET_SIZE - CURRENT_SIZE)) >> "$OUTPUT_FILE"

if [ $? -eq 0 ]; then
  echo "Successfully padded $INPUT_FILE to $OUTPUT_FILE with a size of 64MB."
else
  echo "Error: Padding failed."
  exit 1
fi
