#!/bin/bash

insmod hvisor.ko
mount -t proc proc /proc
mount -t sysfs sysfs /sys
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
nohup ./hvisor virtio start virtio.json &
./hvisor zone start linux2.json && \
cat nohup.out | grep "char device"