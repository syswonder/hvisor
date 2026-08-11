# VirtIO-SCMI 体系说明文档

## 1. 概述

VirtIO-SCMI 是 hvisor 项目中实现的一套虚拟化框架，用于在虚拟机（ZoneU）与主机（Zone0）之间传递 SCMI（System Control and Management Interface）协议消息。通过 VirtIO 传输层，ZoneU 可以安全地访问受控的硬件资源，主要包括：

- **时钟资源（Clock）**：查询和配置系统时钟
- **复位资源（Reset）**：控制复位域

### 1.1 设计目标

在现代异构系统中，不同虚拟机往往需要共享 SoC 上的硬件资源。传统方式是将硬件直通（passthrough）给特定虚拟机，但这会导致：

1. 资源独占：其他虚拟机无法访问
2. 安全隐患：虚拟机直接控制硬件
3. 管理复杂：需要在虚拟机之间手动分配资源

VirtIO-SCMI 通过虚拟化层提供统一的资源访问抽象：
- Zone0（主机）掌握真实的硬件资源
- ZoneU（虚拟机）通过 VirtIO-SCMI 协议请求资源操作
- hvisor 负责消息转发和访问控制

### 1.2 在 hvisor 中的定位

```
┌─────────────────────────────────────────────────────────────────┐
│                         ZoneU (Linux)                           │
│  ┌─────────────┐   ┌─────────────┐     ┌─────────────────────┐  │
│  │ Clock Driver│   │Reset Driver │     │ SCMI VirtIO Driver  │  │
│  │ (SCMI API)  │   │(SCMI API)   │     │ (virtio-mmio)       │  │
│  └──────┬──────┘   └──────┬──────┘     └──────────┬──────────┘  │
└─────────┼─────────────────┼───────────────────────┼─────────────┘
          │                 │                       │
          │    SCMI Protocol Messages               │
          ▼                 ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      hvisor (Hypervisor)                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    VirtIO-SCMI Stack                       │ │
│  │  ┌──────────────┐  ┌─────────────┐  ┌──────────────────┐   │ │
│  │  │ virtio_scmi  │  │ scmi_core   │  │ Protocol Handlers│   │ │
│  │  │ (VirtIO Dev) │◄─┤ (Protocol   │  │ clock.c/reset.c  │   │ │
│  │  │              │  │  Dispatch)  │  │ base.c           │   │ │
│  │  └──────┬───────┘  └──────┬──────┘  └────────┬─────────┘   │ │
│  │         │                 │                  │             │ │
│  │         └─────────────────┴──────────────────┘             │ │
│  │                           │                                │ │
│  │                    Message Dispatch                        │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
          │
          │ ioctl / Hypercall
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Zone0 (Linux)                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   SCMI Server (in hvisor.ko)             │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │   │
│  │  │ Clock Ops   │  │ Reset Ops   │  │ Device Tree     │   │   │
│  │  │ (Linux clk  │  │ (Linux reset│  │ (hvisor node)   │   │   │
│  │  │  framework) │  │  framework) │  │                 │   │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────┘   │   │
│  └─────────┼────────────────┼────────────────────┼──────────┘   │
└────────────┼────────────────┼────────────────────┼──────────────┘
             │                │                    │
             ▼                ▼                    ▼
    ┌──────────────┐  ┌──────────────┐   ┌─────────────────┐
    │ RK3588 Clock │  │ RK3588 Reset │   │ Device Tree     │
    │ Controller   │  │ Controller   │   │ (hvisor node)   │
    └──────────────┘  └──────────────┘   └─────────────────┘
```

---

## 2. 技术背景

### 2.1 SCMI 协议简介

SCMI（System Control and Management Interface）是 ARM 定义的标准化接口，用于：

- **时钟管理**：查询时钟属性、获取/设置时钟频率、启用/禁用时钟
- **复位域管理**：查询复位域属性、执行复位操作
- **电源管理**：处理器功耗状态、器件电源状态
- **性能监控**：性能计数器和配置

SCMI 协议采用 **客户端-服务器** 架构：

- **SCMI Server**：运行在可信固件（ATF）或主机操作系统中，掌握真实硬件
- **SCMI Client**：运行在虚拟机或非安全世界，通过传输层发送请求

#### 2.1.1 SCMI 消息格式

SCMI 协议使用 **打包消息头（Packed Message Header）**：

```
  31          28 27          18 17      10 9    8 7             0
 +--------------+--------------+----------+------+---------------+
 |   Reserved   |   Token ID   |ProtocolID| Type |  Message ID   |
 +--------------+--------------+----------+------+---------------+
```

| 字段 | 位宽 | 说明 |
|------|------|------|
| Message ID | 8 bits | 协议内消息标识 |
| Message Type | 2 bits | 0=Command, 2=Delayed Response, 3=Notification |
| Protocol ID | 8 bits | 协议标识（0x10=Base, 0x11=Power, 0x14=Clock, 0x16=Reset） |
| Token ID | 10 bits | 请求/响应配对标识 |

#### 2.1.2 SCMI 协议列表

ARM SCMI 规范（DEN0056）定义的标准协议如下（hvisor 当前实现 Base/Power Domain/Clock/Reset 四种，其余为预留）：

| 协议 ID | 协议名称 | 功能 | hvisor 支持 |
|---------|----------|------|-------------|
| 0x10 | Base | 版本查询、协议列表、错误通知 | ✅ |
| 0x11 | Power Domain | 电源域属性查询、电源状态控制 | ✅ |
| 0x12 | System Power | 系统级电源状态管理 | — |
| 0x13 | Performance | 性能域管理（DVFS） | — |
| 0x14 | Clock | 时钟属性、频率管理、时钟启用/禁用 | ✅ |
| 0x15 | Sensor | 传感器管理 | — |
| 0x16 | Reset | 复位域属性、复位操作 | ✅ |
| 0x17 | Voltage Domain | 电压域管理 | — |
| 0x18 | Power Capping | 功耗上限管理 | — |
| 0x19 | Pin Control | 引脚控制 | — |
| 0x80-0xFF | Vendor-specific | 厂商自定义协议 | — |

### 2.2 VirtIO 传输层

VirtIO 是 Linux 虚拟化标准框架，提供高效的 guest-to-host 通信机制。

#### 2.2.1 VirtIO-MMIO

VirtIO-MMIO 是基于 MMIO（内存映射 I/O）的传输协议，适合嵌入式场景：

- 特点：无需 PCIe 配置空间，简单易实现
- 适用：ARM SoC、嵌入式系统

VirtIO-MMIO 设备包含：
- **设备特性寄存器**：标识设备支持的功能
- **队列通知**：用于 host/guest 之间的通知机制
- **描述符队列**：存放 I/O 请求和响应缓冲区

#### 2.2.2 VirtIO-SCMI 队列配置

```c
#define SCMI_MAX_QUEUES 2
#define VIRTQUEUE_SCMI_MAX_SIZE 64
#define SCMI_QUEUE_TX 0  // 用于发送请求
#define SCMI_QUEUE_RX 1  // 用于接收响应（当前未使用）
```

### 2.3 为什么需要 VirtIO-SCMI

在 hvisor 虚拟化场景中，VirtIO-SCMI 解决了以下问题：

| 场景 | 问题 | VirtIO-SCMI 解决方案 |
|------|------|----------------------|
| 时钟资源共享 | 多个 ZoneU 需要访问不同时钟 | 通过配置允许列表控制访问权限 |
| 复位隔离 | 误复位关键外设 | 通过资源映射将物理 ID 隔离 |
| 安全隔离 | 防止 ZoneU 直接控制硬件 | 所有操作经过 Zone0 验证执行 |
| 灵活配置 | 不同板卡资源不同 | 支持 JSON 配置映射关系 |

---

## 3. 架构设计

### 3.1 整体架构

VirtIO-SCMI 体系分为三个主要层次：

#### 3.1.1 用户态工具层（hvisor-tool）

位于 `tools/virtio/devices/scmi/`，负责 VirtIO 设备模拟和 SCMI 协议处理：

| 文件 | 功能 |
|------|------|
| [virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c) | VirtIO 设备驱动入口，处理队列中断 |
| [scmi_core.c](../../../tools/virtio/devices/scmi/scmi_core.c) | SCMI 协议核心，注册协议、消息分发 |
| [base.c](../../../tools/virtio/devices/scmi/base.c) | Base 协议实现 |
| [power.c](../../../tools/virtio/devices/scmi/power.c) | Power Domain 协议实现 |
| [clock.c](../../../tools/virtio/devices/scmi/clock.c) | Clock 协议实现 |
| [reset.c](../../../tools/virtio/devices/scmi/reset.c) | Reset 协议实现 |

#### 3.1.2 内核驱动层（scmi_server）

位于 `driver/virtio/scmi/`（[server.c](../../../driver/virtio/scmi/server.c) + 按协议拆分的 [clock.c](../../../driver/virtio/scmi/clock.c) / [reset.c](../../../driver/virtio/scmi/reset.c) / [power.c](../../../driver/virtio/scmi/power.c)），负责实际硬件操作：

- 从 root zone 设备树的 `hvisor_virtio_device` 节点读取物理时钟/复位/电源域资源（`clocks` / `resets` / `power-domains` 属性）
- 执行真实的时钟启用、禁用、频率设置，复位域复位，电源域开关
- 按 ID 缓存对应的 `struct clk *` / `struct reset_control *` 句柄
- 通过 ioctl 与用户态工具通信（`HVISOR_SCMI_CLOCK_IOCTL` / `RESET` / `POWER`）

#### 3.1.3 配置层

- **设备树（DTS，root zone 侧）**：`hvisor_virtio_device` 节点枚举所有可经 SCMI 暴露的物理时钟/复位/电源域资源
- **设备树（DTS，ZoneU 侧）**：定义 `arm,scmi-virtio` 协议节点和 VirtIO 传输通道，设备通过 `scmi_clk` / `scmi_rst` 标签引用资源
- **JSON 配置**：为每个 zone 的 scmi 设备声明允许访问的资源 ID 数组（`clock_ids` / `reset_ids` / `power_ids`）

### 3.2 核心组件

#### 3.2.1 设备抽象（SCMIDev）

`SCMIDev`（定义见 [virtio_scmi.h](../../../tools/virtio/include/virtio_scmi.h)）持有从 JSON 解析出的资源 ID 数组 `clock_ids` / `reset_ids` / `power_ids`（对应数组为 NULL 时表示不支持该协议），以及按设备注册的协议处理器表 `protocols[]`。SCMI 服务器运行在内核态（hvisor.ko），用户态工具通过 ioctl 与之通信。

#### 3.2.2 协议注册机制

协议以**设备内注册表**形式组织：每个协议项包含协议 ID 和消息处理函数，设备创建时由 [virtio.c](../../../tools/virtio/virtio.c) 按 JSON 配置逐协议注册——**BASE 协议恒注册，Clock/Power/Reset 仅在对应 ID 数组非空时注册**。消息分发由 `scmi_handle_message()` 遍历设备注册表完成。

#### 3.2.3 资源映射机制（virtual-ID ABI）

VirtIO-SCMI 采用 **virtual-ID ABI**：JSON 中的 `clock_ids` / `reset_ids` / `power_ids` 数组同时充当**允许列表**和**虚拟→物理映射表**——**数组下标是 ZoneU 看到的虚拟 ID，数组值是对应的物理 ID**。越界的虚拟 ID 会被拒绝并返回 `SCMI_ERR_ENTRY`。

---

## 4. 代码实现分析

### 4.1 消息流程

#### 4.1.1 请求处理流程

```
ZoneU Linux Kernel
      │
      │ (1) SCMI Clock API (e.g., clk_set_rate)
      ▼
VirtIO SCMI Driver (virtio-mmio)
      │
      │ (2) 写入请求到 TX 队列
      ▼
VirtIO Queue Notification
      │
      │ (3) 触发 VM Exit
      ▼
hvisor (VirtIO-SCMI Handler)
      │
      │ (4) virtq_tx_handle_one_request()
      │     - 解析消息头
      │     - 提取 protocol_id, msg_id, token
      ▼
scmi_handle_message()
      │
      │ (5) 根据 protocol_id 分发到对应协议处理器
      ▼
clock.c / reset.c handler
      │
      │ (6) 校验虚拟 ID（clk_phys_id()：下标 → 物理 ID）
      │     无效则返回 SCMI_ERR_ENTRY
      ▼
ioctl(HVISOR_SCMI_CLOCK_IOCTL / HVISOR_SCMI_RESET_IOCTL)
      │
      │ (7) 切换到内核态
      ▼
hvisor.ko（包含 SCMI Server 功能）
      │
      │ (8) 调用 Linux clk/reset 子系统
      ▼
返回结果
```

#### 4.1.2 核心代码解析

- **设备创建与协议注册**（[virtio.c](../../../tools/virtio/virtio.c)）：`scmi_dev_create()` 创建设备，`scmi_dev_parse_clock_ids()` 等从 JSON 解析 ID 数组，再按配置注册协议（BASE 恒注册，其余协议仅在有对应 ID 数组时注册）。
- **请求处理**（[virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c)）：`virtq_tx_handle_one_request()` 从 avail ring 取描述符（`process_descriptor_chain_buf`），校验链布局为 1 读 + 1 写，解析 32 位打包消息头，初始化响应上下文 `scmi_resp_ctx` 后分发；失败路径会**消费并零长度完成描述符**，避免请求永久悬挂；最后按实际写入长度更新 used ring。
- **协议分发**（[scmi_core.c](../../../tools/virtio/devices/scmi/scmi_core.c)）：`scmi_handle_message()` 遍历设备注册表调用对应协议的 handler，未注册协议返回 `SCMI_ERR_SUPPORT`。

### 4.2 资源 ID 解析与校验（virtual-ID ABI）

- **ID 数组解析**（[virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c)）：`parse_id_array()` 将 JSON 的 `clock_ids` / `reset_ids` / `power_ids` 数组解析为 `SCMIDev` 上的 `uint32_t` 数组；未配置或空数组表示协议不支持。
- **虚拟 ID → 物理 ID**（[clock.c](../../../tools/virtio/devices/scmi/clock.c) 的 `clk_phys_id()`）：**数组下标 = ZoneU 看到的虚拟 ID，数组值 = 物理 ID**。例如 `"clock_ids": [0, 1, 5]` 表示 ZoneU 的 clock 0/1/2 分别对应物理 clock 0/1/5；越界下标返回 `SCMI_ERR_ENTRY`。
- 物理 ID 通过 ioctl 传给内核驱动；驱动按 ID 在 `hvisor_virtio_device` 设备树节点中缓存对应的 `struct clk *`。

### 4.3 时钟协议实现

各 handler（[clock.c](../../../tools/virtio/devices/scmi/clock.c)）遵循统一流程：

1. 从请求 payload 解析 `clock_id`；
2. 用 `clk_phys_id()` 校验虚拟 ID 并转换为物理 ID，无效则返回 `SCMI_ERR_ENTRY`；
3. 通过 `hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, ...)` 调用内核驱动（hvisor.ko），携带物理 ID；
4. 用 `scmi_make_response()` 构建响应头 + 状态返回给 ZoneU。

目前异步标志（`flags & 0x1`）暂不支持，返回 `SCMI_ERR_SUPPORT`。

### 4.4 复位协议实现

复位 handler（[reset.c](../../../tools/virtio/devices/scmi/reset.c)）与时钟流程相同：解析 `domain_id` → `rst_phys_id()` 校验并转换 → `hvisor_scmi_ioctl_cmd(HVISOR_SCMI_RESET_IOCTL, ...)` 执行属性查询或复位 → `scmi_make_response()` 返回结果。

---

## 5. 配置指南

### 5.1 设备树配置

ZoneU 的设备树需要定义两部分：

#### 5.1.1 SCMI 协议节点

以本目录 [zone1-linux-npu.dts](zone1-linux-npu.dts) 为例：

```dts
firmware {
    scmi {
        compatible = "arm,scmi-virtio";
        #address-cells = <0x01>;
        #size-cells = <0x00>;
        phandle = <0x1000>;

        scmi_clk: protocol@14 {   // clock protocol
            reg = <0x14>;
            #clock-cells = <0x01>;
        };

        scmi_rst: protocol@16 {   // reset protocol
            reg = <0x16>;
            #reset-cells = <0x01>;
        };

        scmi_pwr: protocol@11 {   // power domain protocol
            reg = <0x11>;
            #power-domain-cells = <0x01>;
        };
    };
};
```

#### 5.1.2 VirtIO 传输通道

以本目录 [zone1-linux-npu.dts](zone1-linux-npu.dts) 为例：

```dts
// virtio-mmio for SCMI
virtio_mmio@ff9c0000 {
    dma-coherent;
    interrupt-parent = <0x01>;
    interrupts = <0x0 0x24 0x1>;
    reg = <0x0 0xff9c0000 0x0 0x200>;
    compatible = "virtio,mmio";
};
```

#### 5.1.3 设备引用 SCMI 资源

设备通过标签直接引用 SCMI 协议节点（以 [zone1-linux-npu.dts](zone1-linux-npu.dts) 的 NPU 节点为例）：

```dts
npu@fdab0000 {
    clocks = <&scmi_clk 0>, <&scmi_clk 1>, <&scmi_clk 2>,
             <&scmi_clk 3>, <&scmi_clk 4>, <&scmi_clk 5>,
             <&scmi_clk 6>, <&scmi_clk 7>;
    resets = <&scmi_rst 0>, <&scmi_rst 1>, <&scmi_rst 2>,
             <&scmi_rst 3>, <&scmi_rst 4>, <&scmi_rst 5>;
};
```

引用中的资源 ID（`<&scmi_clk N>` 的 N）是 ZoneU 侧的**虚拟 ID**，与 JSON 中 `clock_ids` / `reset_ids` 数组的下标对应。

### 5.2 JSON 配置文件

#### 5.2.1 zones-npu-gpu-virtio.json

以本目录 [zones-npu-gpu-virtio.json](zones-npu-gpu-virtio.json) 为例（Zone1 NPU 的 scmi 设备）：

```json
{
    "type": "scmi",
    "addr": "0xff9c0000",
    "len": "0x200",
    "irq": "0x44",
    "status": "enable",
    "clock_ids": [0, 1, 2, 3, 4, 5, 6, 7],
    "reset_ids": [0, 1, 2, 3, 4, 5],
    "power_ids": [0, 1, 2]
}
```

（完整配置见文件本身：Zone1 另含 blk/console/net 设备，Zone2 含独立的 scmi/blk/console/net 设备。）

#### 5.2.2 配置项说明

| 配置项 | 类型 | 说明 |
|--------|------|------|
| `type` | string | 设备类型，固定为 "scmi" |
| `addr` | string | VirtIO-MMIO 基地址 |
| `len` | string | MMIO 区域大小 |
| `irq` | string/int | 中断号 |
| `status` | string | "enable" / "disable" |
| `clock_ids` | array | 允许的时钟 ID 列表；**数组下标 = ZoneU 虚拟 ID，值 = 物理 ID** |
| `reset_ids` | array | 允许的复位 ID 列表（同上语义） |
| `power_ids` | array | 允许的电源域 ID 列表（同上语义） |
| `clock_count` / `reset_count` / `power_count` | 内部 | 由数组长度自动推导；**配置了对应数组才注册对应协议** |

> 未配置 `clock_ids`（或为空）时，该设备不注册 Clock 协议（`dev->clock_ids = NULL`）。

### 5.3 资源 ID 示例

以本目录 [zones-npu-gpu-virtio.json](zones-npu-gpu-virtio.json) 的真实配置为例：

**Zone1（NPU）的 scmi 设备**——虚拟 ID 与物理 ID 相同（恒等映射）：

```json
"clock_ids": [0, 1, 2, 3, 4, 5, 6, 7],
"reset_ids": [0, 1, 2, 3, 4, 5],
"power_ids": [0, 1, 2]
```

**Zone2（GPU/显示）的 scmi 设备**——虚拟 ID 从 0 连续编号，物理 ID 允许非连续/分段（例如 clock 23/24 对应物理 clock 35/36）：

```json
"clock_ids": [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
              24, 25, 26, 27, 28, 29, 35, 36],
"reset_ids": [6, 7, 8, 9, 10, 11, 12, 13, 16, 17, 18, 19, 20, 21, 22],
"power_ids": [3, 4, 5]
```

即 Zone2 的 clock 0 = 物理 clock 8，clock 1 = 物理 clock 9，……，clock 23 = 物理 clock 35。越界的虚拟 ID（如请求 clock 24）会被 `clk_phys_id()` 拒绝并返回 `SCMI_ERR_ENTRY`，ZoneU 无法访问列表之外的资源。

---

## 6. 使用示例

### 6.1 基于 RK3588 的完整配置（Zone1 NPU + Zone2 GPU/VOP/HDMI）

完整的配置文件位于 [examples/rk3588-aarch64/with_virtio_scmi/](../../../examples/rk3588-aarch64/with_virtio_scmi/)

#### 6.1.1 目录结构

```
examples/rk3588-aarch64/with_virtio_scmi/
├── zones-npu-gpu-virtio.json   # 双 zone VirtIO 设备配置（Zone1 NPU + Zone2 GPU）
├── zone0.dts                   # Root zone 设备树（含 hvisor_virtio_device 节点）
├── zone1-linux-npu.json        # Zone1 (NPU) 配置
├── zone1-linux-npu.dts         # Zone1 设备树（NPU + 保留 DMA 池）
├── zone2-linux-gpu-hdmi.json   # Zone2 (GPU + VOP + HDMI 显示) 配置
├── zone2-linux-gpu-hdmi.dts    # Zone2 设备树（GPU、VOP2、HDMI0、hdmiphy、VOP IOMMU）
└── README.md                   # 本文档
```

#### 6.1.2 Zone 布局与资源

| Zone | CPU | 设备 | SCMI 资源 |
|------|-----|------|-----------|
| **Zone1 (npu)** | 6, 7 | RKNPU、virtio-blk/net/console | clocks 0-7, resets 0-5, power 0-2 |
| **Zone2 (gpu)** | 4, 5 | Mali GPU、VOP2、HDMI0、HD-PHY、VOP IOMMU | clocks 8-29, 35-36, resets 6-13, 16-22, power 3-5 |

#### 6.1.3 构建和运行

1. 编译 hvisor（启用 SCMI 支持）：
   ```bash
   make VIRTIO_SCMI=y KDIR=/path/to/linux-kernel
   ```

2. 加载驱动：
   ```bash
   insmod driver/hvisor.ko
   ```

3. 启动 hvisor 并加载配置：

   首先启动 VirtIO 守护进程（后台运行），使用双 zone 配置：
   ```bash
   ./hvisor virtio start examples/rk3588-aarch64/with_virtio_scmi/zones-npu-gpu-virtio.json &
   ```

   然后依次启动两个 Zone：
   ```bash
   ./hvisor zone start examples/rk3588-aarch64/with_virtio_scmi/zone1-linux-npu.json
   ./hvisor zone start examples/rk3588-aarch64/with_virtio_scmi/zone2-linux-gpu-hdmi.json
   ```

   部署时需将 `Image`、`zone0.dtb`、`zone1-linux-npu.dtb`、`zone2-linux-gpu-hdmi.dtb` 以及配置中引用的 rootfs 镜像放到 hvisor-tool 的运行目录。

### 6.2 常见使用场景

#### 6.2.1 场景一：Zone1 NPU 时钟管理

目标：让 ZoneU 中的 NPU 驱动正常工作（参考 [zone1-linux-npu.dts](zone1-linux-npu.dts) 与 [zones-npu-gpu-virtio.json](zones-npu-gpu-virtio.json)）

配置步骤：
1. 在设备树中，NPU 节点的 clocks 指向 scmi_clk
2. 在 zones-npu-gpu-virtio.json 中为 scmi 设备配置对应时钟 ID
3. ZoneU 启动后，Linux SCMI 驱动会自动发现时钟控制器

```dts
/* 设备树（zone1-linux-npu.dts） */
npu@fdab0000 {
    clocks = <&scmi_clk 0>, <&scmi_clk 1>, <&scmi_clk 2>,
             <&scmi_clk 3>, <&scmi_clk 4>, <&scmi_clk 5>,
             <&scmi_clk 6>, <&scmi_clk 7>;
};
```

```json
/* zones-npu-gpu-virtio.json（Zone1 的 scmi 设备） */
"clock_ids": [0, 1, 2, 3, 4, 5, 6, 7]
```

#### 6.2.2 场景二：NPU 复位控制

目标：允许 ZoneU 复位 NPU 控制器

配置步骤：
1. 在设备树中，NPU 节点的 resets 指向 scmi_rst
2. 在 zones-npu-gpu-virtio.json 中通过 reset_ids 配置允许的复位域

```dts
/* 设备树（zone1-linux-npu.dts） */
npu@fdab0000 {
    resets = <&scmi_rst 0>, <&scmi_rst 1>, <&scmi_rst 2>,
             <&scmi_rst 3>, <&scmi_rst 4>, <&scmi_rst 5>;
};
```

```json
/* zones-npu-gpu-virtio.json（Zone1 的 scmi 设备） */
"reset_ids": [0, 1, 2, 3, 4, 5]
```

### 6.3 调试方法

#### 6.3.1 启用调试日志

[virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c) 中已内置请求日志，启用 `log_debug` 级别后可见：

```c
log_debug("SCMI request: protocol=0x%x, msg=0x%x, type=%d, token=0x%x",
          hdr->protocol_id, hdr->msg_id, hdr->msg_type, hdr->token);
```

#### 6.3.2 常见问题排查

| 问题 | 可能原因 | 排查方法 |
|------|----------|----------|
| 时钟操作返回 -2 (SCMI_ERR_ENTRY) | clock_id 越界（不在允许列表） | 检查 zones-npu-gpu-virtio.json 的 clock_ids |
| 复位操作无效 | reset_id 越界（不在允许列表） | 检查 zones-npu-gpu-virtio.json 的 reset_ids |
| VirtIO 队列无响应 | 中断未正确配置 | 检查中断号和 GIC 配置 |
| ioctl 失败 | 内核驱动未加载 | 检查 hvisor.ko 是否加载（SCMI 功能包含在其中） |

#### 6.3.3 内核调试

查看内核日志：
```bash
dmesg | grep -E "scmi|hvisor"
```

---

## 7. 扩展开发

### 7.1 添加新协议

VirtIO-SCMI 框架支持扩展新的 SCMI 协议。Power Domain 协议（0x11）已作为一个完整参考实现包含在 `power.c` 中。如需添加其他协议（如 Performance 0x13、Sensor 0x15 等），可遵循以下步骤：

1. **定义协议 ID 和消息 ID**（virtio_scmi.h）

2. **创建协议文件**，参考 `power.c` 的实现模式：
   - 实现 handle_VERSION、handle_PROTOCOL_ATTRIBUTES 等通用消息
   - 实现协议特定的消息处理函数
   - 通过 `hvisor_scmi_ioctl_cmd()` 与内核模块通信
   - 使用 `scmi_dev_register_protocol()` 注册到设备

3. **添加内核模块支持**（driver/virtio/scmi/）：
   - 定义 ioctl 命令号和参数结构（driver/virtio/scmi/server.h）
   - 实现内核侧的硬件操作（driver/virtio/scmi/ 下按协议拆分的文件）
   - 声明 ioctl 处理函数（server.h）
   - 在 hvisor_main.c 中注册 ioctl 处理

4. **注册协议**（virtio.c）：设备创建时用 `scmi_dev_register_protocol()` 注册（BASE 恒注册，其余协议按 ID 数组是否配置决定）。

5. **解析配置**（virtio.c）：在 `VirtioTSCMI` 分支中解析 JSON 的 `clock_ids` / `reset_ids` / `power_ids` 字段，调用 `scmi_dev_parse_clock_ids()` 等存入 `SCMIDev`。

### 7.2 性能优化

1. **缓存优化**：clock.c 中已实现 clock_count 缓存
2. **批量操作**：支持 Describe Rates 批量查询
3. **异步通知**：预留异步响应支持

---

## 8. 参考资料

- [ARM SCMI 协议规范](https://developer.arm.com/documentation/den0056/latest)
- [VirtIO 规范](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html)
- [hvisor 项目](https://github.com/syswonder/hvisor)
- Linux 内核时钟子系统：Documentation/driver-api/clk.rst
- Linux 内核复位子系统：Documentation/driver-api/reset.rst

---

## 附录 A：SCMI 消息 ID 参考

### Base 协议（0x10）

| Message ID | 名称 | 说明 |
|------------|------|------|
| 0x0 | VERSION | 获取协议版本 |
| 0x1 | PROTOCOL_ATTRIBUTES | 获取协议属性 |
| 0x2 | MESSAGE_ATTRIBUTES | 获取消息属性 |
| 0x3 | DISCOVER_VENDOR | 获取厂商信息 |
| 0x7 | DISCOVER_AGENT | 获取代理信息 |

### Clock 协议（0x14）

| Message ID | 名称 | 说明 |
|------------|------|------|
| 0x0 | VERSION | 获取协议版本 |
| 0x1 | PROTOCOL_ATTRIBUTES | 获取协议属性 |
| 0x2 | MESSAGE_ATTRIBUTES | 获取消息属性 |
| 0x3 | CLOCK_ATTRIBUTES | 获取时钟属性 |
| 0x4 | DESCRIBE_RATES | 描述支持的频率 |
| 0x5 | RATE_SET | 设置频率 |
| 0x6 | RATE_GET | 获取当前频率 |
| 0x7 | CONFIG_SET | 启用/禁用时钟 |
| 0x8 | CONFIG_GET | 获取时钟状态 |
| 0x9 | NAME_GET | 获取时钟名称 |

### Reset 协议（0x16）

| Message ID | 名称 | 说明 |
|------------|------|------|
| 0x0 | VERSION | 获取协议版本 |
| 0x1 | PROTOCOL_ATTRIBUTES | 获取协议属性 |
| 0x2 | MESSAGE_ATTRIBUTES | 获取消息属性 |
| 0x3 | RESET_ATTRIBUTES | 获取复位域属性 |
| 0x4 | RESET | 执行复位 |
| 0x5 | RESET_NOTIFY | 复位通知 |

---

## 附录 B：错误码参考

| 错误码 | 值 | 说明 |
|--------|-----|------|
| SCMI_SUCCESS | 0 | 成功 |
| SCMI_ERR_SUPPORT | -1 | 不支持 |
| SCMI_ERR_PARAMS | -2 | 无效参数 |
| SCMI_ERR_ACCESS | -3 | 访问被拒绝 |
| SCMI_ERR_ENTRY | -4 | 未找到 |
| SCMI_ERR_RANGE | -5 | 值超出范围 |
| SCMI_ERR_BUSY | -6 | 设备忙 |
| SCMI_ERR_COMMS | -7 | 通信错误 |
| SCMI_ERR_GENERIC | -8 | 通用错误 |
| SCMI_ERR_HARDWARE | -9 | 硬件错误 |
| SCMI_ERR_PROTOCOL | -10 | 协议错误 |
