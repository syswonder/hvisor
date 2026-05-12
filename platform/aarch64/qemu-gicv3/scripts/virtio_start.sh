#!/bin/bash

mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
./hvisor virtio start zone1-linux-virtio.json &