#!/bin/bash

insmod hvisor.ko 2>/dev/null || true
mount -t proc proc /proc 2>/dev/null || true
mount -t sysfs sysfs /sys 2>/dev/null || true
rm -f nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts 2>/dev/null || true
./hvisor virtio start zone1-linux-virtio.json &
./hvisor zone start zone1-linux.json
