#!/bin/bash

insmod hvisor.ko
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
./hvisor virtio start zone1-linux-virtio.json &
./hvisor zone start zone1-linux.json