本目录下的配置文件，用于在QEMU aarch64下启动hvisor，并启动一个linux作为zone1，zone1使用hvisor-tool提供的Virtio disk和Virtio console作为自己的磁盘和终端设备。进入Root Linux后，可参考如下命令启动Zone1：
```
insmod hvisor.ko
mount -t proc proc /proc
mount -t sysfs sysfs /sys
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start zone1_linux.json && \
cat nohup.out | grep "char device" && \
script /dev/null
```