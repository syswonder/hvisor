# insmod hvisor.ko
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
./hvisor virtio start virtio-backend.json &
./hvisor zone start zone1-linux-virtio.json && \
# cat /var/log/syslog | grep "char device" && \
script /dev/null
