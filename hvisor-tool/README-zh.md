# README
README：[中文](./README-zh.md) | [English](./README.md)

本仓库包含附属于[hvisor](https://github.com/syswonder/hvisor)的命令行工具及内核模块，命令行工具中还包含了Virtio守护进程，用于提供Virtio设备。命令行工具以及内核模块需要单独编译后，在管理虚拟机root linux-zone0上使用。整个仓库结构为：

```
hvisor-tool
	-tools: 包含命令行工具以及Virtio守护进程
	-driver: hvisor对应的内核模块
	-examples: 不同运行环境下的配置文件示例
```

## 编译步骤

以下操作均在x86主机的目录`hvisor-tool`下，进行交叉编译。

* 编译命令行工具及内核模块

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux VIRTIO_GPU=[y/n]
```

其中，`<arch>`应该为`arm64`和`riscv`之一。

`<log>`为`LOG_TRACE`、`LOG_DEBUG`、`LOG_INFO`、`LOG_WARN`、`LOG_ERROR`、`LOG_FATAL`之一，用来控制Virtio守护进程的日志输出等级。

`/path/to/your-linux`为root linux的kernel源码目录。具体的编译选项请见[Makefile](./Makefile)、[tools/Makefile](./tools/Makefile)、[driver/Makefile](./driver/Makefile)。

例如，要编译面向`arm64`的命令行工具，可以执行：

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

即可在`tools/hvisor`和`driver/hvisor.ko`，将其复制到root linux的根文件系统，使用即可。

## 使用步骤

### 内核模块

使用命令行工具、Virtio守护进程之前，需要在zone0上加载内核模块，便于用户态程序与Hypervisor进行交互：

```
insmod hvisor.ko
```

卸载内核模块的操作为：

```
rmmod hvisor.ko
```

### 命令行工具

在root linux-zone0中，使用命令行工具可以创建、关闭其他虚拟机。

* 启动新的虚拟机

  hvisor-tool通过一个配置文件启动一个新的虚拟机：

  ```
  ./hvisor zone start <vm_config.json>
  ```

  `<vm_config.json>`是描述一个虚拟机配置的文件，例如：

  * [在QEMU-aarch64上启动zone1](./examples/qemu-aarch64/with_virtio_blk_console/zone1_linux.json)：可参考[这里](./examples/qemu-aarch64/with_virtio_blk_console/README.md)了解如何配置并在zone1上启动一个linux。

  * [在NXP-aarch64上启动zone1](./examples/nxp-aarch64/zone1_linux.json)：使用该文件直接启动zone1时，需首先启动Virtio守护进程，对应的配置文件为[virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json)。

* 关闭id为1的虚拟机：

```
./hvisor zone shutdown -id 1
```

* 打印当前所有虚拟机的信息：

```
./hvisor zone list
```

### Virtio守护进程

Virtio守护进程可为虚拟机提供Virtio MMIO设备，目前支持四种设备：Virtio-blk、Virtio-net、Virtio-console和Virtio-gpu设备。

#### 前置条件

要使用Virtio守护进程，需要在**Root Linux的设备树**中增加一个名为`hvisor_virtio_device`的节点，例如：

```dts
hvisor_virtio_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```

这样，当hvisor向Root Linux注入中断号为`32+0x20`的中断时，便会进入`hvisor.ko`中注册的中断处理函数，唤醒Virtio守护进程。

如果`32+0x20`号中断已经被某个设备占用了，除了修改上述设备树节点，还需要修改hvisor主仓库中`IRQ_WAKEUP_VIRTIO_DEVICE`变量。对arm来说，如果`interrupts`中填写的值为`0xa`则`IRQ_WAKEUP_VIRTIO_DEVICE`需要为`32+0xa`；如果是riscv架构，则不需要增加32，使二者相等即可。

#### Virtio设备的启动和创建

在Root Linux上，执行以下示例指令：

```c
// 注意要先启动守护进程，再启动各个zones
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start <vm_config.json>
```

其中`nohup ... &`说明该命令会创建一个守护进程，且该进程的日志输出保存在当前文件夹下的nohup.out文件中。

`virtio_cfg.json`则是一个描述Virtio设备的JSON文件，例如[virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json)。该示例文件会依次执行：

1. 地址空间映射

首先将id为1的虚拟机`zone1`的RAM内存区域通过mmap映射到Virtio守护进程的地址空间。

2. 创建Virtio-blk设备

创建一个Virtio-blk设备，`zone1`会通过一片MMIO区域与该设备通信，这片MMIO区域的起始地址为`0xa003c00`，长度为`0x200`。同时设置设备中断号为78，对应磁盘镜像为`rootfs2.ext4`。

3. 创建Virtio-console设备

创建一个Virtio-console设备，用于`zone1`主串口的输出。root linux需要执行`screen /dev/pts/x`命令进入该虚拟控制台，其中`x`可通过nohup.out日志文件查看。

如要退回到主控制台，按下快捷键`ctrl+a+d`。如果在qemu中，则需要按下`ctrl+a ctrl+a+d`。如要再次进入虚拟控制台，执行`screen -r [SID]`，其中SID为该screen会话的进程ID。

4. 创建Virtio-net设备

由于`net`设备的`status`属性为`disable`，因此不会创建Virtio-net设备。如果`net`设备的`status`属性为`enable`，那么会创建一个Virtio-net设备，MMIO区域的起始地址为`0xa003600`，长度为`0x200`，设备中断号为75，MAC地址为`00:16:3e:10:10:10`，由id为1的虚拟机使用，连接到名为`tap0`的Tap设备。

5. 创建Virtio-gpu设备

要使用virtio-gpu设备，需要在hvisor-tool编译命令中加入`VIRTIO_GPU=y`字段，同时还需安装`libdrm`并进行其他配置，具体请见[hvisor-book](https://hvisor.syswonder.org/chap04/subchap03/VirtIO/GPUDevice.html)和[配置文件示例](./examples/qemu-aarch64/with_virtio_gpu/README.md)。配置文件中如果`gpu`设备`status`属性为`enable`，则会创建一个 Virtio-gpu 设备，其 MMIO 区域从 `0xa003400` 开始，长度为 `0x200`，中断号为 74。默认的扫描输出(scanout)尺寸为宽度 `1280px`，高度 `800px`。

#### 关闭Virtio设备

执行该命令即可关闭Virtio守护进程及所有创建的设备：

```
pkill hvisor-virtio
```
