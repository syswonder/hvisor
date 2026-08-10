su root # passwd: bianbu

insmod hvisor.ko

nohup ./hvisor virtio start virtio-backend.json &
./hvisor zone start zone1-linux-virtio.json

./hvisor zone list

screen /dev/pts/0