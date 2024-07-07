# hvisor 
<p align = "center">
<br><br>
<img src="https://img.shields.io/badge/hvisor-orange" />
<img src="https://img.shields.io/github/license/syswonder/hvisor?color=red" />
<img src="https://img.shields.io/github/contributors/syswonder/hvisor?color=blue" />
<img src="https://img.shields.io/github/languages/code-size/syswonder/hvisor?color=green">
<img src="https://img.shields.io/github/repo-size/syswonder/hvisor?color=white">
<img src="https://img.shields.io/github/languages/top/syswonder/hvisor?color=orange">
<br><br>
</p>

README：[中文](./README-zh.md) | [English](README.md)

基于Linux的Armv8 Rust Hypervisor，部分借鉴了[RVM1.5](https://github.com/rcore-os/RVM1.5)和[jailhouse](https://github.com/siemens/jailhouse)。

## 进展

- [x] Architecture: aarch64
- [x] Platform: Qemu virt aarch64
- [x] Exception
- [x] Gicv3
- [x] Memory
- [x] Enable non root linux
- [ ] VirtIO device: block, net
- [ ] Architecture: riscv64
- [ ] Platform: nxp

## 如何运行

详细的配置和运行教程，包括配置开发环境、制作根文件系统等，请参考：[详细配置流程](https://report.syswonder.org/#/2023/20230421_ARM64-QEMU-jailhouse)。

为了降低入门开发难度，[云盘](https://bhpan.buaa.edu.cn/link/AA1BF35BBB05DA40EB8A837C2B2B3C8277)（提取码：sysH）提供了已经编译好的Linux内核`Image`和根文件系统`ubuntu-20.04-rootfs_ext4.img`，用户名为arm64，密码为一个空格。其主目录下的目录组织如下：

```
├── home
	├── arm64 
        ├── images: 包含一个Linux Image和内存文件系统
        ├── hvisor: 运行hvisor所需要的文件
        ├── jailhouse: 运行jailhouse所需要的文件
```

下面介绍基于`ubuntu-20.04-rootfs_ext4.img`，在jailhouse/hvisor上运行一个non-root-linux的方法：

1. 在本项目目录下构建`rvmarm.bin`：

   ```makefile
   make all
   ```

   并将`target/aarch64/debug/rvmarm.bin`复制到`ubuntu-20.04-rootfs_ext4.img`中的`hvisor`目录下。

2. 在本项目目录下启动qemu

   ```bash
   sudo qemu-system-aarch64 \
       -machine virt,gic_version=3 \
       -machine virtualization=true \
       -cpu cortex-a57 \
       -machine type=virt \
       -nographic \
       -smp 16  \
       -m 1024 \
       -kernel your-linux-Image-path/Image \
       -append "console=ttyAMA0 root=/dev/vda rw mem=768m" \
       -drive if=none,file=your-rootfs-path/ubuntu-20.04-rootfs_ext4.img,id=hd0,format=raw \
       -device virtio-blk-device,drive=hd0 \
       -net nic \
       -net user,hostfwd=tcp::2333-:22
   ```

3. 启动后输入用户名`arm64`，密码为一个空格。

4. 进入主目录，启动non-root-linux：

   * hvisor：进入hvisor文件夹，依次执行：

     ```
     ./setup.sh
     ./linux.sh
     ```

   * jailhouse：进入jailhouse文件夹，执行：

     ```
     ./linux.sh
     ```

### 运行双串口

如果希望non-root-linux和root-linux处于两个不同的终端中，可以在qemu启动命令的最后加入：

```
-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3
```

启动qemu后，观察到终端输出的`char device redirected to /dev/pts/num (label serial3)`信息，在另一个终端中执行：

```
sudo screen /dev/pts/num
```

其中num为一个具体的数字。
