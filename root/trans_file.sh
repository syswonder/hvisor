#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path-to-file>"
    exit 1
fi

file_path=$1

# 解析 Makefile 中的 ARCH 变量
ARCH_VALUE=$(make -pn | awk -F ' = ' '/^ARCH =/ {print $2; exit}')
# 判断 ARCH 是否为 riscv 或 arm64
if [ "$ARCH_VALUE" == "riscv" ]; then
    disk_path="../hvisor/images/riscv64/virtdisk"
elif [ "$ARCH_VALUE" == "arm64" ]; then
    disk_path="../hvisor/images/aarch64/virtdisk"
else
    echo "unknown ARCH"
    exit 1
fi

# 检查文件是否存在
if [ ! -f "$file_path" ]; then
    echo "Error: File '$file_path' not found."
    exit 1
fi

sudo mount "$disk_path"/rootfs1.ext4 "$disk_path"/rootfs
if [ "$ARCH_VALUE" == "riscv" ]; then
    sudo cp "$file_path" "$disk_path"/rootfs/home/riscv64
elif [ "$ARCH_VALUE" == "arm64" ]; then
    sudo cp "$file_path" "$disk_path"/rootfs/home/arm64
else
    echo "unknown ARCH"
    exit 1
fi

if [ $? -eq 0 ]; then
    echo "File has been successfully copied"
else
    echo "Error: Failed to copy the file."
    exit 1
fi
sudo umount "$disk_path"/rootfs
