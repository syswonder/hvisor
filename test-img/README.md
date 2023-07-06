# 测试环境说明
Image来自yjy、cxy编译的linux内核，内核版本为5.4.0。
配套的jailhouse.ko，config文件已经打过RVM1.5的patch，
rootfs来自buildroot
## 启动流程
qemu启动命令为

```sh
qemu-system-aarch64 \
-drive file=./rootfs.qcow2,discard=unmap,if=none,id=disk,format=qcow2 \
-device virtio-blk-device,drive=disk \
-m 1G -serial mon:stdio  \
-kernel Image \
-append "root=/dev/vda mem=768M"  \
-cpu cortex-a57 -smp 4 -nographic -machine virt,gic-version=3,virtualization=on \
-device virtio-serial-device -device virtconsole,chardev=con \
-chardev vc,id=con  \
-net nic \
-net user,hostfwd=tcp::2333-:22
```

qemu启动后，将guest内的文件传入guest linux中
```sh
scp -P 2333 -r test-img/guest/* root@localhost:~/
```
将hypervisor的镜像传入guest
```sh
make scp
```
## 各组件编译说明
可参考文档[https://github.com/syswonder/report/blob/main/docs/2023/20230421_ARM64-QEMU-jailhouse.md]
下面也给出功能较为简单的编译流程

### 交叉编译工具
在x86机器上编译arm64程序需要使用交叉编译工具，这里用gcc-aarch64-linux-gnu
```sh
sudo apt-cache search aarch64   #查看可安装的版本
sudo apt-get install gcc-aarch64-linux-gnu    #安装一个默认的版本
```
### kernel编译
以5.4.0为例
```sh
wget https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.4.tar.xz
tar -xvJf linux-5.4.tar.xz
cd linux-5.4
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc)
```

### rootfs编译
使用buildroot工具，它可以生成一个完成的linux系统，不过我们这里只用它生成根文件系统。

>buildroot可以自动化为嵌入式系统建造一个完整和可引导的Linux，Buildroot主要用于小型或嵌入式系统，Buildroot可以自动建造所需要的交叉编译工具链，创建根文件系统，编译一个Linux内核映像，并为目标嵌入式系统生成引导装载器。
```sh
wget https://buildroot.org/downloads/buildroot-2023.02.3.tar.xz
tar -xvJf buildroot-2023.02.3.tar.xz
cd buildroot-2023.02.3
make menuconfig
```
在menuconfig界面需要进行以下设置：

```
Target Architecture (AArch64 (little endian)) 
Target Architecture Variant (cortex-A57)
(root) Root password
(ttyAMA0) TTY port
[*] dhcp (ISC)
[*]   dhcp client
[*] dhcpcd
[*] dropbear
[*] ext2/3/4 root filesystem                       
    ext2/3/4 variant (ext4) 
(1G)  exact size
```
配置完成后直接make就可以，所生成的文件位于output/images
```
sudo make
cd output/images
```
其中的rootfs.ext4即是生成的文件系统，大小为1G，与之前的```(1G)  exact size```相同，这决定文件系统的初始容量，可以自定义设置，里面实际占用的大小没有1G，为了节省空间，我们可以把它转化为qcow2格式。
```sh
qemu-img convert -O qcow2 rootfs.ext4 rootfs.qcow2
```

## jailhouse编译
jailhouse也可以交叉编译，不过需要先有编译好的内核工程，另外本项目中对jailhouse进行了一些修改，需要先打上patch,具体命令如下：
```sh
git clone https://github.com/siemens/jailhouse.git
cd jailhouse
git checkout v0.10
patch -f -p1 < path/to/jailhouse.patch 
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KDIR=path/to/kernel -j$(nproc)
```
编译完成后，在各个文件夹下可以看见对应的生成文件，其中：
- configs/arm64里的xxx.cell文件是vm配置文件，在enable和cell create时使用
- driver/jailhouse.ko 是加载到root cell的内核模块，提供部分管理功能，通过ioctl操作为用户态提供接口，使用hypercall调用hypervisor提供的接口。
- hypervisor/jailhouse.bin 为jailhouse的hypervisor镜像，运行在EL2，提供虚拟化的核心功能，本项目的sysHyper主要就是实现对它的替换。
- tools/jailhouse是root cell的用户态程序，用来调用内核模块jailhouse.ko，是面向用户的管理接口。
- inmates 里面是一些可运行的demo