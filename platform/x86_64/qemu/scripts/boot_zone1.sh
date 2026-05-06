#!/bin/bash

insmod hvisor.ko
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start ./zone1_linux.json