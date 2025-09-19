insmod hvisor.ko
mount -t proc /proc /proc
mount -t sysfs /sys /sys
./hvisor zone start zone1-linux.json
