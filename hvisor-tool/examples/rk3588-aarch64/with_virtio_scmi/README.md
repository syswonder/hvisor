# VirtIO-SCMI 框架使用指南

本手册介绍如何在 hvisor 环境下配置并使用 VirtIO-SCMI 框架。该框架通过消息传递机制，使虚拟机（ZoneU）能够安全地访问受控的时钟（Clock）和复位（Reset）资源。

## 1. 现状说明

VirtIO-SCMI 核心逻辑已适配至 hvisor-tool 的最新架构，兼容`config v5`版本。设备代码位于 `devices/scmi/` 目录，与其他 VirtIO 设备保持一致的目录结构。该版本包含完整的消息转发与协议解析功能，支持通过配置文件定义资源访问权限和映射关系，足以支持直通设备的初始化与基本资源管理。

## 2. 虚拟机（ZoneU）配置

### 2.1 设备树（DTS）修改

ZoneU 的设备树需要定义通信通道、协议语义，并重定向硬件资源引用。

- **SCMI 协议节点 (firmware/arm_scmi)**：
  定义具体的传输协议语义。需指定其作为 clock-controller 和 reset-controller。

```c
firmware {
    scmi {
        compatible = "arm,scmi-virtio";   // 使用 virtio 通道传输
        /* 这里的 address/size 对应协议 ID 寻址 */
        #address-cells = <1>;
        #size-cells = <0>;

        /* 时钟协议：负责处理所有通过 SCMI 转发的时钟请求 */
        scmi_clk: protocol@14 {
            reg = <0x14>;                // 协议 ID: 0x14 (Clock Management)
            #clock-cells = <1>;          // 参数为时钟 ID
            phandle = <0x02>;            // 这里指定了一个虚拟clock controller节点，供其他设备引用
        };

        /* 复位协议：负责处理所有重置域请求 */
        scmi_reset: protocol@16 {
            reg = <0x16>;                // 协议 ID: 0x16 (Reset Domain Management)
            #reset-cells = <1>;          // 参数为复位 ID
            phandle = <0x1002>;          // 指定虚拟reset controller节点，注意 phandle 需全局唯一，不可与时钟节点重复
        };
    };
};
```

- **VirtIO 传输节点 (virtio@xxxx)**：
  定义底层的 VirtIO-MMIO 通道，用于承载 SCMI 报文。

```c
// virtio scmi 传输通道
virtio_mmio@ff9c0000 {
    compatible = "virtio,mmio";
    reg = <0x0 0xff9c0000 0x0 0x200>;   // MMIO 地址，需确保不与其他设备重叠
    interrupt-parent = <0x01>;          // 引用 GIC 中断控制器
    interrupts = <0x0 0x2b 0x1>;        // 使用新的中断号 (如 0x2b)
    dma-coherent;
};
```

- **直通 SoC 设备修改**：
  对于需要直通的 SoC 外设（如 GMAC、USB），需修改其 clocks 和 resets 属性：
  - 将属性中的 provider phandle 由原始物理控制器修改为上述的 SCMI 节点，其他无需修改
  - 此时，SCMI 节点在 ZoneU 视角下充当统一的资源供给单元。

```c
	ethernet@fe1c0000 {
		// ...

		clocks = <0x02 0x144 0x02 0x145 0x02 0x168 0x02 0x16d 0x02 0x143>; // 这里的0x02指向上述SCMI协议虚拟出来的clock-controller节点
		clock-names = "stmmaceth\0clk_mac_ref\0pclk_mac\0aclk_mac\0ptp_ref";
		resets = <0x1002 0x138>;        // 这里的0x1002指向上述SCMI协议虚拟出来的reset-controller节点
		reset-names = "stmmaceth";

		// ...
	}
```

### 2.2 Hvisor 配置文件修改

除了修改设备树，还需要调整虚拟机对应的 Hvisor 配置文件，以确保传输通道在底层被正确分配。

- **config.json (虚拟机配置)**：
  需要在对应的 ZoneU 配置中增加 SCMI 设备相关的资源描述。新增以下内容：
  
```json
{
    "memory_regions": [
        // ...
        {
            "type": "virtio",
            "physical_start": "0xff9c0000",
            "virtual_start":  "0xff9c0000",
            "size": "0x1000"
        },
        // ...
    ],
    // ...
}
```
- **virtio_cfg.json (VirtIO 配置)**：
  在该文件中手动添加 SCMI 设备的 VirtIO 区域信息，并配置允许的资源和映射关系。
  
```json
{
    "zones": [
        {
            "id": 1,
            // ...
            "devices": [
                    // ...
		            {
		                "type": "scmi",
		                "addr": "0xff9c0000",
		                "len": "0x200",
		                "irq": 75,
		                "status": "enable",
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

**配置说明**：
- `allowed_list`：指定允许虚拟机访问的资源 ID 列表
  - `reset_ids`：允许的复位域 ID 列表，例如 `[0]` 表示只允许访问 ID 为 0 的复位域
  - `clock_ids`：允许的时钟 ID 列表，使用 `["*"]` 表示允许访问所有时钟
- `reset_map`：指定虚拟复位 ID 到物理复位 ID 的映射关系，例如 `{"0": 523}` 表示虚拟 ID 0 映射到物理 ID 523
- `clock_phandle`：指定时钟控制器的 phandle 值，对应设备树中 SCMI 时钟协议节点的 phandle，例如 `2`
- `reset_phandle`：指定复位控制器的 phandle 值，对应设备树中 SCMI 复位协议节点的 phandle，例如 `2`

### 2.3 Linux 内核要求

- **版本支持**：
  - 若内核版本高于 5.15，通常已内置 VirtIO-SCMI 驱动支持。
  - 若内核版本低于 5.15，必须打入 Backport Patch 以引入驱动支持（请联系作者获取）。

- **Reset 后端逻辑**：
  部分复位操作的后端逻辑目前暂留在内核代码中。后续计划将其迁移至外置内核模块（hvisor.ko）。如需自定义复位逻辑，请联系作者。

- **源码获取**：
  如需适配好的内核源码，请单独联系作者。

## 3. Hypervisor (hvisor) 配置

无需任何修改。
hvisor 在此框架中仅作为透明的消息转发层（Trampoline），不参与协议解析。直接使用现有主分支代码即可支持 VirtIO-SCMI 通信。

## 4. 后续工作

计划进一步完善 SCMI 协议的功能，包括支持更多的协议消息类型和优化资源管理机制。

## 5. 调试建议

- **确认节点挂载**：启动后检查 `/sys/kernel/debug/clk/clk_summary` 是否出现 SCMI 挂载的时钟节点。