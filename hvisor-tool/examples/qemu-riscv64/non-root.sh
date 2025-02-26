#!/bin/bash

cd /
sudo mount -t proc proc /proc
cd /home/riscv64
sudo insmod hvisor.ko

if grep -q APLIC /proc/interrupts; then
    sudo ./hvisor zone start linux2-aia.json
else
    sudo ./hvisor zone start linux2.json
fi
