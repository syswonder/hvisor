本目录存放在nxp imx8mp上运行hvisor所需的各样配置文件。
其中Root Linux设备树为linux1.dts，设备树中包含hdmi、gpu等用于显示的节点。
Non Root Linux设备树为linux2.dts，启动Non Root Linux的命令如下：
```
insmod hvisor.ko
rm nohup.out
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
nohup ./hvisor virtio start con_virtio.json &
./hvisor zone start linux2.json && \
cat nohup.out | grep "char device" && \
script /dev/null && \
screen /dev/pts/0
```