# insmod hvisor.ko
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
nohup ./hvisor virtio start zone1-linux-virtio.json &
./hvisor zone start zone1-linux.json && \
cat nohup.out | grep "char device" && \
script /dev/null
