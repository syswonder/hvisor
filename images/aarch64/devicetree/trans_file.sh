#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path-to-file>"
    exit 1
fi

file_path=$1
disk_path="../virtdisk"
# 检查文件是否存在
if [ ! -f "$file_path" ]; then
    echo "Error: File '$file_path' not found."
    exit 1
fi

sudo mount "$disk_path"/rootfs1.ext4 "$disk_path"/rootfs
sudo cp "$file_path" "$disk_path"/rootfs/home/arm64/

if [ $? -eq 0 ]; then
    echo "File has been successfully copied"
else
    echo "Error: Failed to copy the file."
    exit 1
fi
sudo umount "$disk_path"/rootfs
