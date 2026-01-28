# insmod hvisor.ko
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
# # exec rsyslog
# rsyslogd
# start zone1
./hvisor virtio start zone1-linux-virtio.json &
./hvisor zone start zone1-linux.json && \
# grep "char device" /var/log/syslog && \
script /dev/null
