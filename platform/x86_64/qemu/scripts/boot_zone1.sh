#!/bin/bash

insmod hvisor.ko
nohup ./hvisor virtio start virtio_cfg.json &
sleep 3
./hvisor zone start ./zone1_linux.json