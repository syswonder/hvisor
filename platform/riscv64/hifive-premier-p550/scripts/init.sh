#!/bin/bash

mount -t proc proc /proc
mount -t sysfs sys /sys
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts -o gid=5,mode=620,ptmxmode=0666

# Set one temp hostname, maybe rockos-eswin
hostname $(cat /etc/hostname)

# insmod related ko
insmod /home/debian/module-6.77/eic7700_dsp.ko
insmod /home/debian/module-6.77/eic7700_npu.ko

while true; do
    /sbin/getty -L ttyS0 115200 vt100
done