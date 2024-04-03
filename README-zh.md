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

一款轻量级Type-1虚拟机监控器，使用Rust语言编写。

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

**启动qemu**

在 `hvisor` 目录下，执行：

```
make run
```

**观察终端输出信息**

```
char device redirected to /dev/pts/num (label serial)
```

这里的 `num` 是一个具体的数字，记住它。

**在uboot中输入启动命令**

该启动命令会从物理地址`0x40400000`启动hvisor，设备树的地址为`0x40000000`

```
bootm 0x40400000 - 0x40000000
```

hvisor启动时，会自动启动root linux（用于管理的Linux），并进入root linux的shell界面。

**启动non-root-linux**

在root linux中，进入`/home`目录，执行脚本：

```
./start-linux.sh
```

在宿主机上输入以下指令，启动另一个终端，用于第二个Linux的输出：

```
sudo screen /dev/pts/num
```

`num` 是上一步在root终端中输出的具体数字。

**验证地址空间的不同**

现在启动了两个终端，可以通过以下命令验证两个内核使用了不同的地址空间。

```shell
cat /proc/iomem
```
