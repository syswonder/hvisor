#!/bin/bash

insmod hvisor.ko
nohup ./hvisor virtio start virtio_cfg_asterinas.json &
sleep 3
./hvisor zone start ./zone1-asterinas.json
