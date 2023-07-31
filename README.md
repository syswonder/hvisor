# sysHyper 
![Static Badge](https://img.shields.io/badge/sysHyper-orange)
![GitHub](https://img.shields.io/github/license/syswonder/sysHyper?color=red)

[![Contributors](https://img.shields.io/github/contributors/syswonder/sysHyper?color=blue)](https://github.com/syswonder/sysHyper)
![GitHub Repo stars](https://img.shields.io/github/stars/syswonder/sysHyper?color=yellow)
![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/w/syswonder/sysHyper?color=black)

![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/syswonder/sysHyper?color=green)
![GitHub repo size](https://img.shields.io/github/repo-size/syswonder/sysHyper?color=white)
![GitHub top language](https://img.shields.io/github/languages/top/syswonder/sysHyper?color=orange)




Armv8 hypervisor based on Linux & implemented in Rust，porting from [RVM1.5](https://github.com/rcore-os/RVM1.5) & [jailhouse](https://github.com/siemens/jailhouse)

## Progress
- [x] arch_entry
- [x] cpu
- [x] logging
- [x] exception
- [x] gicv3
- [x] memory
- [ ] ....
## Platform
- [x] qemu
- [ ] imx
- [ ] ti
- [ ] rpi4
## 环境配置
### 安装rust
首先安装 Rust 版本管理器 rustup 和 Rust 包管理器 cargo，为了在国内加速访问，可以设置使用中科大的镜像服务器。
```sh
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
curl https://sh.rustup.rs -sSf | sh  
```
最好把 Rust 包管理器 cargo 镜像地址 crates.io 也替换成中国科学技术大学的镜像服务器，来加速三方库的下载。 打开或新建 ~/.cargo/config 文件，并把内容修改为：
```sh
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
replace-with = 'ustc'
[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```
### qemu模拟器编译
```sh
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 ninja-build  # 安装编译所需的依赖包
wget https://download.qemu.org/qemu-7.0.0.tar.xz  # 下载源码
tar xvJf qemu-7.0.0.tar.xz   # 解压
cd qemu-7.0.0
./configure   #生成设置文件
make -j$(nproc)   #编译
qemu-system-aarch64 --version   #查看版本
```
qemu版本>7.2需要额外配置，否则在启动时可能出现以下问题
```
network backend user is not compiled into this binary
```
需要在编译前进行以下设置：
```sh
sudo apt install libslirp-dev 
../configure --enable-slirp
```
编译完成后可以```sudo make install```将 Qemu 安装到 ```/usr/local/bin``` 目录下,
也可以编辑``` ~/.bashrc``` 文件（如果使用的是默认的 bash 终端），在文件的末尾加入：
```
export PATH=$PATH:/path/to/qemu-7.0.0/build
```

### 启动qemu
```sh
mkdir qemu-test    # 新建一个文件夹用来测试
git submodule update --init --recursive    # 更新子模块
cp -r test-img/* qemu-test   #将所需的文件传入测试文件夹
cd qemu-test/host
./test.sh    #启动qemu
```
linux默认用户密码为root/root
### 编译sysHyper
在host执行
```sh
make     #编译得到hypervisor镜像rvmarm.bin
make scp   #将得到的rvmarm.bin文件传入qemu上运行的linux
```
### 运行sysHyper
将必要的文件传入guest linux：
```sh
scp -P 2333 -r qemu-test/guest/* root@localhost:~/
```
在guest linux中
```sh
./setup.sh  #设置文件路径
./enable.sh   #运行sysHyper，开启虚拟化
cat /proc/cpuinfo   #查看当前linux cpuinfo
jailhouse cell create configs/qemu-arm64-gic-demo.cell  #新建一个cell，将cpu 3 移出root cell
cat /proc/cpuinfo   #查看当前linux cpuinfo，cpu3被shutdown了
jailhouse disable  # 关闭虚拟化
```
### output
应该可以看到hypervisor运行打印的一些信息


### 调试
可以使用vscode进行可视化调试，在原有qemu命令末尾加上```-s -S```
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
-net user,hostfwd=tcp::2333-:22 -s -S
```
先启动qemu，然后按F5即可开始调试

### 原版jailhouse
在开发调试的过程中，为了方便与原版jailhouse做对比，还提供了v0.12版本的原版jailhouse运行环境：
- test-img/host/jail-img 内核
- test-img/guest/jail   原版jailhouse编译生成文件
运行命令为：
```sh
qemu-system-aarch64 \
-drive file=./rootfs.qcow2,discard=unmap,if=none,id=disk,format=qcow2 \
-m 1G -serial mon:stdio -netdev user,id=net,hostfwd=tcp::23333-:22 \
-kernel jail-img \
-append "root=/dev/vda mem=768M"  \
-cpu cortex-a57 -smp 16 -nographic -machine virt,gic-version=3,virtualization=on \
-device virtio-serial-device -device virtconsole,chardev=con -chardev vc,id=con -device virtio-blk-device,drive=disk \
-device virtio-net-device,netdev=net
```
在guest中：
```sh
cd jail
insmod ./jailhouse.ko
cp jailhouse.bin /lib/firmware/
./jailhouse enable configs/qemu-arm64.cell
```


本项目的相关文档在
https://github.com/syswonder/report
