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
│                         ZoneU (Linux)                          │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐  │
│  │ Clock Driver│   │Reset Driver │   │ SCMI VirtIO Driver  │  │
│  │ (SCMI API)  │   │(SCMI API)   │   │ (virtio-mmio)       │  │
│  └──────┬──────┘   └──────┬──────┘   └──────────┬──────────┘  │
└─────────┼─────────────────┼───────────────────────┼─────────────┘
          │                 │                       │
          │    SCMI Protocol Messages             │
          ▼                 ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      hvisor (Hypervisor)                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    VirtIO-SCMI Stack                        │ │
│  │  ┌──────────────┐  ┌─────────────┐  ┌──────────────────┐ │ │
│  │  │ virtio_scmi  │  │ scmi_core   │  │ Protocol Handlers│ │ │
│  │  │ (VirtIO Dev) │◄─┤ (Protocol   │  │ clock.c/reset.c  │ │ │
│  │  │              │  │  Dispatch)  │  │ base.c            │ │ │
│  │  └──────┬───────┘  └──────┬──────┘  └────────┬─────────┘ │ │
│  │         │                  │                   │          │ │
│  │         └──────────────────┴───────────────────┘          │ │
│  │                         │                                   │ │
│  │                    Message Dispatch                         │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
          │
          │ ioctl / Hypercall
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Zone0 (Linux)                              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              SCMI Server (集成在 hvisor.ko 中)              │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │   │
│  │  │ Clock Ops   │  │ Reset Ops   │  │ Device Tree      │  │   │
│  │  │ (Linux clk  │  │ (Linux reset│  │ (phandle lookup)│  │   │
│  │  │  framework) │  │  framework) │  │                 │  │   │
│  │  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘  │   │
│  └─────────┼────────────────┼────────────────────┼───────────┘   │
└────────────┼────────────────┼────────────────────┼───────────────┘
             │                │                    │
             ▼                ▼                    ▼
    ┌──────────────┐  ┌──────────────┐   ┌─────────────────┐
    │  RK3588 Clock │  │ RK3588 Reset │   │ Device Tree     │
    │  Controller  │  │ Controller    │   │ (phandle→node)  │
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
| Protocol ID | 8 bits | 协议标识（0x10=Base, 0x14=Clock, 0x16=Reset） |
| Token ID | 10 bits | 请求/响应配对标识 |

#### 2.1.2 支持的协议

| 协议 ID | 协议名称 | 功能 |
|---------|----------|------|
| 0x10 | Base | 版本查询、协议列表、错误通知 |
| 0x14 | Clock | 时钟属性、频率管理、时钟启用/禁用 |
| 0x16 | Reset | 复位域属性、复位操作 |

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
| [clock.c](../../../tools/virtio/devices/scmi/clock.c) | Clock 协议实现 |
| [reset.c](../../../tools/virtio/devices/scmi/reset.c) | Reset 协议实现 |
| [base.c](../../../tools/virtio/devices/scmi/base.c) | Base 协议实现 |

#### 3.1.2 内核驱动层（scmi_server）

位于 `driver/scmi_server.c`，负责实际硬件操作：

- 获取 Linux 内核的时钟/复位子系统句柄
- 执行真实的时钟启用、禁用、频率设置
- 执行复位域的复位操作
- 通过 ioctl 与用户态工具通信

#### 3.1.3 配置层

- **设备树（DTS）**：定义 SCMI 协议节点和 VirtIO 传输通道
- **JSON 配置**：定义资源映射和访问权限

### 3.2 核心组件

#### 3.2.1 设备抽象（SCMIDev）

```c
typedef struct virtio_scmi_dev {
    int fd;
} SCMIDev;
```

每个 VirtIO-SCMI 设备实例持有文件描述符，用于与内核驱动通信。

#### 3.2.2 协议注册机制

```c
struct scmi_protocol {
    uint8_t id;                    // 协议 ID
    
    // 消息处理函数
    int (*handle_request)(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                         const struct iovec *req_iov, struct iovec *resp_iov);
};
```

用户态工具启动时注册协议处理器：

```c
extern int virtio_scmi_base_init(void);   // 注册 Base 协议
extern int virtio_scmi_clock_init(void);  // 注册 Clock 协议
extern int virtio_scmi_reset_init(void);  // 注册 Reset 协议
```

#### 3.2.3 资源映射机制

VirtIO-SCMI 使用两层映射实现资源隔离：

1. **允许列表（allowed_ids）**：定义哪些 SCMI ID 可以被访问
2. **ID 映射（map）**：将虚拟 SCMI ID 映射到物理硬件 ID

```c
typedef struct {
    uint32_t scmi_id;    // 虚拟机看到的 ID
    uint32_t phys_id;   // 实际硬件 ID
} scmi_map_entry_t;

typedef struct {
    scmi_map_entry_t *map;       // 映射表
    uint32_t map_count;          // 映射条目数
    uint32_t *allowed_ids;      // 允许的 ID 列表
    uint32_t allowed_count;     // 允许的 ID 数量
    bool allow_all;             // 是否允许所有 ID
} scmi_map_context_t;
```

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
      │ (6) 验证 clock_id/reset_id 合法性
      │     通过 scmi_map_id() 转换为物理 ID
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

**VirtIO 设备初始化**（[virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c#L17-L30)）：

```c
SCMIDev *init_scmi_dev() {
    SCMIDev *dev = (SCMIDev *)malloc(sizeof(SCMIDev));
    if (dev) {
        dev->fd = -1;
        // 注册支持的协议
        virtio_scmi_base_init();
        virtio_scmi_clock_init();
        virtio_scmi_reset_init();
    }
    return dev;
}
```

**请求处理**（[virtio_scmi.c](../../../tools/virtio/devices/scmi/virtio_scmi.c#L32-L90)）：

```c
static int virtq_tx_handle_one_request(void *dev, VirtQueue *vq) {
    // 1. 获取描述符链
    int ret = process_descriptor_chain(vq, &desc_idx, &iov, &flags, ...);
    
    // 2. 解析 SCMI 消息头
    uint32_t packed_header = *(uint32_t *)iov[0].iov_base;
    uint8_t protocol_id = SCMI_PROTOCOL_ID(packed_header);
    uint8_t msg_id = SCMI_MSG_ID(packed_header);
    uint16_t token = SCMI_TOKEN_ID(packed_header);
    
    // 3. 分发到协议处理器
    int status = scmi_handle_message(dev, protocol_id, msg_id, token,
                                   &iov[0], &iov[1]);
    
    // 4. 更新 VirtIO Used Ring
    update_used_ring(vq, desc_idx, iov[0].iov_len + iov[1].iov_len);
    return 0;
}
```

**协议分发**（[scmi_core.c](../../../tools/virtio/devices/scmi/scmi_core.c#L10-L30)）：

```c
const struct scmi_protocol *scmi_get_protocol_by_id(uint8_t protocol_id) {
    for (int i = 0; i < protocol_count; i++) {
        if (protocols[i]->id == protocol_id) {
            return protocols[i];
        }
    }
    return NULL;
}
```

### 4.2 资源映射实现

#### 4.2.1 映射初始化

从 JSON 配置读取映射关系（[scmi_core.c](../../../tools/virtio/devices/scmi/scmi_core.c#L76-L130)）：

```c
int scmi_init_map(scmi_map_context_t *ctx, void *allowed_list_json, 
                  void *map_json, const char *id_key, const char *map_key) {
    // 1. 解析允许列表
    if (ids_json) {
        ctx->allowed_count = SAFE_CJSON_GET_ARRAY_SIZE(ids_json);
        // 支持通配符 "*" 表示允许所有
        if (strcmp(id_json->valuestring, "*") == 0) {
            ctx->allow_all = true;
        }
    }
    
    // 2. 解析映射表
    if (map) {
        ctx->map_count = cJSON_GetArraySize(map);
        // 将 JSON 键值对转换为 scmi_map_entry_t
    }
}
```

#### 4.2.2 ID 验证与映射

```c
bool scmi_is_valid_id(scmi_map_context_t *ctx, uint32_t id) {
    if (ctx->allow_all) return true;
    // 检查 ID 是否在允许列表中
    for (uint32_t i = 0; i < ctx->allowed_count; i++) {
        if (ctx->allowed_ids[i] == id) return true;
    }
    return false;
}

uint32_t scmi_map_id(scmi_map_context_t *ctx, uint32_t scmi_id) {
    // 在映射表中查找
    for (uint32_t i = 0; i < ctx->map_count; i++) {
        if (ctx->map[i].scmi_id == scmi_id) {
            return ctx->map[i].phys_id;
        }
    }
    // 未找到则返回原值（相同映射）
    return scmi_id;
}
```

### 4.3 时钟协议实现

#### 4.3.1 时钟属性查询

```c
static int handle_clock_clock_attributes(SCMIDev *dev, uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov) {
    // 1. 获取请求中的 clock_id
    uint32_t clock_id = *(uint32_t *)req->payload;
    
    // 2. 验证 clock_id 是否在允许列表中
    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }
    
    // 3. 转换为物理 clock ID
    uint32_t phys_clk_id = scmi_map_id(&clock_map_ctx, clock_id);
    
    // 4. 通过 ioctl 调用内核驱动
    struct hvisor_scmi_clock_args args;
    args.subcmd = HVISOR_SCMI_CLOCK_GET_NAME;
    args.u.clock_name_info.clock_id = phys_clk_id;
    ioctl(fd, HVISOR_SCMI_CLOCK_IOCTL, &args);
    
    // 5. 构建响应
    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}
```

#### 4.3.2 时钟频率设置

```c
static int handle_clock_rate_set(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct iovec *resp_iov) {
    struct clock_rate_set_info *req = (void *)req->payload;
    
    // 1. 验证并映射 clock_id
    if (!is_valid_clock_id(req->clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }
    uint32_t phys_clk_id = scmi_map_id(&clock_map_ctx, req->clock_id);
    
    // 2. 调用内核驱动设置频率
    struct hvisor_scmi_clock_args args;
    args.subcmd = HVISOR_SCMI_CLOCK_RATE_SET;
    args.u.clock_rate_set_info.clock_id = phys_clk_id;
    args.u.clock_rate_set_info.rate = req->rate;
    hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_RATE_SET, &args, sizeof(args));
    
    // 3. 返回结果
    scmi_make_response(dev, token, resp_iov, args.ret);
}
```

### 4.4 复位协议实现

#### 4.4.1 复位域属性

```c
static int handle_reset_attributes(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct iovec *resp_iov) {
    uint32_t domain_id = *(uint32_t *)req->payload;
    
    // 1. 验证并映射 reset_id
    if (!is_valid_reset_id(domain_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }
    uint32_t phys_rst_id = scmi_map_id(&reset_map_ctx, domain_id);
    
    // 2. 构建响应
    struct scmi_msg_resp_reset_attributes *attr = resp->payload;
    attr->attributes = 0;
    attr->reset_latency = 100;  // 100ms
    snprintf(attr->reset_name, 15, "rst_%u", phys_rst_id);
    
    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}
```

#### 4.4.2 执行复位

```c
static int handle_reset(SCMIDev *dev, uint16_t token,
                        const struct iovec *req_iov,
                        struct iovec *resp_iov) {
    struct scmi_msg_req_reset *req = (void *)req->payload;
    
    // 1. 验证并映射 reset_id
    uint32_t phys_rst_id = scmi_map_id(&reset_map_ctx, req->domain_id);
    
    // 2. 调用内核驱动执行复位
    struct hvisor_scmi_reset_args args;
    args.subcmd = HVISOR_SCMI_RESET_RESET;
    args.u.reset_info.reset_id = phys_rst_id;
    args.u.reset_info.flags = req->flags;
    args.u.reset_info.reset_state = req->reset_state;
    hvisor_scmi_ioctl(HVISOR_SCMI_RESET_RESET, &args, sizeof(args));
    
    // 3. 返回结果
    scmi_make_response(dev, token, resp_iov, args.ret);
}
```

---

## 5. 配置指南

### 5.1 设备树配置

ZoneU 的设备树需要定义两部分：

#### 5.1.1 SCMI 协议节点

```dts
firmware {
    scmi {
        compatible = "arm,scmi-virtio";
        #address-cells = <1>;
        #size-cells = <0>;
        phandle = <0x1000>;

        /* 时钟协议 - Protocol ID 0x14 */
        protocol@14 {
            reg = <0x14>;
            #clock-cells = <1>;
            phandle = <0x02>;
        };

        /* 复位协议 - Protocol ID 0x16 */
        protocol@16 {
            reg = <0x16>;
            #reset-cells = <1>;
            phandle = <0x1002>;
        };
    };
};
```

#### 5.1.2 VirtIO 传输通道

```dts
/* VirtIO-SCMI 传输通道 */
virtio_mmio@ff9c0000 {
    compatible = "virtio,mmio";
    reg = <0x0 0xff9c0000 0x0 0x200>;
    interrupt-parent = <0x01>;    // 引用 GIC 中断控制器
    interrupts = <0x0 0x2b 0x1>;  // 中断号 = 0x2b
    dma-coherent;
};
```

#### 5.1.3 设备引用 SCMI 资源

需要使用 SCMI 资源的设备，通过 phandle 值直接引用（注意：ZoneU 设备树中的 phandle 值应与 SCMI 协议节点定义一致）：

```dts
ethernet@fe1c0000 {
    /* 时钟引用：0x02 是 scmi_clk 的 phandle */
    clocks = <0x02 0x144>, <0x02 0x145>, <0x02 0x168>;
    clock-names = "stmmaceth", "clk_mac_ref", "pclk_mac";
    
    /* 复位引用：0x1002 是 scmi_reset 的 phandle */
    resets = <0x1002 0x0>;
    reset-names = "stmmaceth";
};
```

> **注意**：如果希望使用标签引用（如 `&scmi_clk`），需要在设备树中通过别名定义：
> ```dts
> / {
>     scmi_clk = &scmi_clock_node;
>     scmi_reset = &scmi_reset_node;
> };
> ```
> 但更常见的方式是直接使用 phandle 数值。

### 5.2 JSON 配置文件

#### 5.2.1 virtio_cfg.json

```json
{
    "zones": [
        {
            "id": 1,
            "devices": [
                {
                    "type": "scmi",
                    "addr": "0xff9c0000",
                    "len": "0x200",
                    "irq": 75,
                    "status": "enable",
                    "clock_phandle": 2,
                    "reset_phandle": 2,
                    "clock_max_num": 721,
                    "reset_max_num": 1,
                    "allowed_list": {
                        "reset_ids": [0],
                        "clock_ids": ["*"]
                    },
                    "reset_map": {
                        "0": 523
                    }
                }
            ]
        }
    ]
}
```

#### 5.2.2 配置项说明

| 配置项 | 类型 | 说明 |
|--------|------|------|
| `type` | string | 设备类型，固定为 "scmi" |
| `addr` | string | VirtIO-MMIO 基地址 |
| `len` | string | MMIO 区域大小 |
| `irq` | int | 中断号 |
| `clock_phandle` | int | 时钟 provider 的设备树 phandle |
| `reset_phandle` | int | 复位 provider 的设备树 phandle |
| `clock_max_num` | int | 最大时钟数量 |
| `reset_max_num` | int | 最大复位域数量 |
| `allowed_list` | object | 允许访问的资源列表 |
| `allowed_list.clock_ids` | array | 允许的时钟 ID 列表，"*" 表示所有 |
| `allowed_list.reset_ids` | array | 允许的复位 ID 列表，"*" 表示所有 |
| `clock_map` | object | 时钟 ID 映射表（ZoneU ID → 物理 ID） |
| `reset_map` | object | 复位 ID 映射表（ZoneU ID → 物理 ID） |

### 5.3 资源映射示例

以 RK3588 为例：

```json
/* 
 * 时钟 ID 映射示例
 * ZoneU 看到的 clock ID 10 
 * 实际映射到物理 clock ID 144
 * 这样 ZoneU 无法直接访问物理 ID
 */
"clock_map": {
    "10": 144
}

/* 
 * 复位 ID 映射示例
 * ZoneU 看到的 reset ID 0 
 * 实际映射到物理 reset ID 523
 * 这样 ZoneU 无法直接访问其他复位域
 */
"reset_map": {
    "0": 523
}

/*
 * 时钟允许所有 ID（*）
 * 但通过映射层可以限制具体访问
 */
"clock_ids": ["*"]
```

---

## 6. 使用示例

### 6.1 基于 RK3588 的完整配置

完整的配置文件位于 [examples/rk3588-aarch64/with_virtio_scmi/](../../../examples/rk3588-aarch64/with_virtio_scmi/)

#### 6.1.1 目录结构

```
examples/rk3588-aarch64/with_virtio_scmi/
├── virtio_cfg.json       # VirtIO 设备配置
├── zone1-linux.json      # Zone1 (Linux) 配置
├── zone1-linux.dts       # Zone1 设备树
└── README.md             # 本文档
```

#### 6.1.2 构建和运行

1. 编译 hvisor（启用 SCMI 支持）：
   ```bash
   make VIRTIO_SCMI=y KDIR=/path/to/linux-kernel
   ```

2. 加载驱动：
   ```bash
   insmod driver/hvisor.ko
   ```

3. 启动 hvisor 并加载配置：

   首先启动 VirtIO 守护进程（后台运行）：
   ```bash
   nohup ./hvisor virtio start examples/rk3588-aarch64/with_virtio_scmi/virtio_cfg.json &
   ```

   然后启动 ZoneU：
   ```bash
   ./hvisor zone start examples/rk3588-aarch64/with_virtio_scmi/zone1-linux.json
   ```

### 6.2 常见使用场景

#### 6.2.1 场景一：网络设备时钟管理

目标：让 ZoneU 中的以太网驱动正常工作

配置步骤：
1. 在设备树中，以太网节点的 clocks 指向 scmi_clk
2. 在 virtio_cfg.json 中允许对应时钟 ID
3. ZoneU 启动后，Linux SCMI 驱动会自动发现时钟控制器

```dts
/* 设备树 */
&ethernet {
    clocks = <0x02 0x144>, <0x02 0x145>;
};
```

```json
/* virtio_cfg.json */
"allowed_list": {
    "clock_ids": [0x144, 0x145, 0x168, 0x16d, 0x143]
}
```

#### 6.2.2 场景二：USB 复位控制

目标：允许 ZoneU 复位 USB 控制器

配置步骤：
1. 在设备树中，USB 节点的 resets 指向 scmi_reset
2. 在 virtio_cfg.json 中配置允许的复位 ID 和映射

```dts
/* 设备树 */
&usb {
    resets = <0x1002 0x0>;
};
```

```json
/* virtio_cfg.json */
"allowed_list": {
    "reset_ids": [0]
},
"reset_map": {
    "0": 523  /* 映射到物理复位域 523 */
}
```

### 6.3 调试方法

#### 6.3.1 启用调试日志

在代码中添加调试输出：

```c
// 在 virtio_scmi.c 中
log_debug("SCMI request: protocol=0x%x, msg=0x%x, token=0x%x",
          protocol_id, msg_id, token);
```

#### 6.3.2 常见问题排查

| 问题 | 可能原因 | 排查方法 |
|------|----------|----------|
| 时钟操作返回 -2 (SCMI_ERR_ENTRY) | clock_id 不在允许列表 | 检查 virtio_cfg.json 的 allowed_list |
| 复位操作无效 | phandle 配置错误 | 检查 device tree 的 clock_phandle/reset_phandle |
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

VirtIO-SCMI 框架支持扩展新的 SCMI 协议。以添加电源管理协议（0x11）为例：

1. **定义协议 ID**（virtio_scmi.h）：
   ```c
   #define SCMI_PROTO_ID_POWER   0x11
   ```

2. **创建协议文件**（power.c）：
   ```c
   static int handle_power_protocol_attributes(...) { ... }
   static int handle_power_domain_attributes(...) { ... }
   static int handle_power_state_set(...) { ... }
   static int handle_power_state_get(...) { ... }
   
   static const struct scmi_protocol power_protocol = {
       .id = SCMI_PROTO_ID_POWER,
       .handle_request = handle_power_request,
   };
   
   int virtio_scmi_power_init(void) {
       return scmi_register_protocol(&power_protocol);
   }
   ```

3. **注册协议**（virtio_scmi.c）：
   ```c
   extern int virtio_scmi_power_init(void);
   // 在 init_scmi_dev() 中添加
   virtio_scmi_power_init();
   ```

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