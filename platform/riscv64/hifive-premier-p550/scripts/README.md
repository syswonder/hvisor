
Note: `boot_zone1.sh` is an example usage with `hvisor-tool`

`boot_zone1.sh` will start virtio backend and boot zone1, please don't create backend repeatedly.

If you want to reattch the same one virtio-console, please use `screen -r`.

One example is below:

```bash
# boot zone1, in this script insmod hvisor.ko
./boot_zone1.sh

# list all zones
./hvisor zone list

# attach zone1's virtio-console, then use ctrl+a+d to return to zone0's terminal
screen -S zone1 /dev/pts/0 

# shutdown zone1
./hvisor zone shutdown -id 1

# restart zone1
./hvisor zone start zone1-linux.json

# reattach to virtio-console
screen -r zone1
```
<br>

---

<br>

If you want to test npu, you can use these cmds.

```bash
# in hvisor
insmod /lib/modules/6.6.77-win2030/kernel/drivers/soc/eswin/ai_driver/dsp/eic7700_dsp.ko 
insmod /lib/modules/6.6.77-win2030/kernel/drivers/soc/eswin/ai_driver/npu/eic7700_npu.ko
# these two modules are needed
cd /home/debian/qwen
/opt/eswin/sample-code/npu_sample/qwen_sample/bin/es_qwen2 ./config.json


# in phys linux
cd qwen
sudo /opt/eswin/sample-code/npu_sample/qwen_sample/bin/es_qwen2 ./config.json
```

Other, if you see:
```bash
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
```

You can execute cmds:
```bash
mount -t proc proc /proc
mount -t sysfs sys /sys
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts -o gid=5,mode=620,ptmxmode=0666

# Set one temp hostname, maybe rockos-eswin
hostname $(cat /etc/hostname)

# insmod related ko
insmod eic7700_dsp.ko
insmod eic7700_npu.ko

# execute getty
/sbin/getty -L ttyS0 115200 vt100
# enter user & passwd

# execute qwen example
cd qwen
sudo /opt/eswin/sample-code/npu_sample/qwen_sample/bin/es_qwen2 ./config.json
# enter passwd: debian
```