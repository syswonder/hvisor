# insmod hvisor.ko
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
nohup ./hvisor virtio start virtio-backend.json &
./hvisor zone start zone1-linux-virtio.json && \
cat nohup.out | grep "char device" && \
script /dev/null
