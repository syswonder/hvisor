
尝试精简ok6254.dts设备树文件,并给出dts中保留的相关节点的解释

设备树精简一般首先保留cpu、interrupt-controller、mmc（一般可以只保留mmc0）、timer、串口、pinctrl等基础内容

并保留相关保留节点的依赖的设备树节点


可以参考zone0.dts等已经精简过的设备树进行裁剪

一般来说，需要保留的节点有

```
memory
cpus
reserved-memory 可以删掉部分内容，并将hvisor所占用的地址添加进去
psci
pmu
timer 如果无法确定，建议在设备树中搜索“fixed-clock”，“timer”关键字，查一查他们的含义，
将对应的内容全部添加到zone0设备树中，除非确定某些fixed-clock是用于支持未添加的设备的。
clock-controller
interrupt-controller 注释掉其中ITS的部分
pinctrl 可以不改
mmc 保留mmc0 mmc1
firmware
serail 串口2 串口3
chosen linux启动参数
hvisor_virtio_device hvisor-tool额外添加的
```

如果发现一些设备树条目会引出一大堆内容，但是其并非时钟中断等关键内容，则可尝试将条目注释
------



// 简单总线 连接多个子设备 定义了地址空间和范围，允许子设备通过该总线访问硬件资源
	bus@f0000 {
		compatible = "simple-bus";
		// 地址和大小的单元数 2个32位单元
		#address-cells = <0x02>;
		#size-cells = <0x02>;
		// 总线地址空间的映射关系，描述了从总线地址到物理地址的映射范围
        // 子节点的地址可以直接用于访问硬件资源，而无需通过 ranges 属性进行地址转换
		ranges;
		// 子总线
		bus@4000000 {
			compatible = "simple-bus";
			#address-cells = <0x02>;
			#size-cells = <0x02>;
			// 表示从子总线地址 0x4000000 开始，映射到物理地址 0x4000000，大小为 0x1ff1400 字节
			
			ranges = <0x00 0x4000000 0x00 0x4000000 0x00 0x1ff1400>;
			phandle = <0x51>;


-------


interrupt-controller@4210000 {
				// 表示该中断控制器与 TI 的 SCI（System Control Interface）中断管理器兼容
				compatible = "ti,sci-intr";
				// 中断控制器的寄存器地址范围
				reg = <0x00 0x4210000 0x00 0x200>;
				// 指定中断触发类型，0x01 通常表示边沿触发
				ti,intr-trigger-type = <0x01>;
				//该节点是一个中断控制器
				interrupt-controller;
				// 指定该中断控制器的父中断控制器，<0x01> 通常是 GIC
				interrupt-parent = <0x01>;
				// 中断单元的数量
				#interrupt-cells = <0x01>;
				// 指定与 SCI 通信的接口
				ti,sci = <0x05>;
				// 断控制器的设备 ID
				ti,sci-dev-id = <0x05>;
				//  0x00 开始，长度为 0x68，每个中断占用 4 个单元
				ti,interrupt-ranges = <0x00 0x68 0x04>;
				phandle = <0x06>;
			};

-------

解析 interrupts = <0x00 0xb2 0x04>;
0x00

表示中断类型。0x00 通常表示 SPI（Shared Peripheral Interrupt），即共享外设中断。
0xb2

表示中断号。0xb2 是设备的中断号，由中断控制器管理。
0x04

表示中断触发类型。0x04 通常表示边沿触发（edge-triggered）。

中断号 0xb2 是由 interrupt-controller@1800000 分配和管理的。设备通过中断号向中断控制器注册中断请求。

中断处理流程
当设备触发中断时：

中断信号会发送到 interrupt-controller@1800000。
中断控制器根据中断号 0xb2 将中断分发给对应的处理器核心。
处理器核心根据中断号执行相应的中断服务程序（ISR）。
设备树中的关联
在设备树中，interrupt-parent 属性通常用于指定设备的中断控制器。例如：

这里的 <0x01> 引用了 interrupt-controller@1800000 的 phandle，表示该设备的中断由 interrupt-controller@1800000 管理。


----

interrupt-controller@a00000 是 gpio@600000 和 gpio@601000 的中断父节点。
gpio@600000 和 gpio@601000 是 gt928_ts@14 和 tsc2007@48 的中断父节点。

--------------

1. CPU 和缓存相关节点
/cpus: 定义 CPU 核心及其属性。
cpu@0: ARM Cortex-A53 核心 0。
cpu@1: ARM Cortex-A53 核心 1。
l2-cache0: L2 缓存节点。
2. 内存相关节点
memory@80000000: 定义系统内存的物理地址和大小。
reserved-memory: 定义预留内存区域。
nonroot@b0000000: 非根域内存。
tfa@9e780000: TFA（Trusted Firmware-A）内存。
optee@9e800000: OP-TEE 内存。
r5f-dma-memory@a0000000: R5F 核心的 DMA 内存。
lpm-memory@a1000000: 低功耗模式内存。
m4f-dma-memory@a4000000: M4F 核心的 DMA 内存。
m4f-memory@a4100000: M4F 核心的共享内存。
3. 中断控制器
interrupt-controller@1800000: ARM GICv3 中断控制器。
msi-controller@1820000: GICv3 ITS（Interrupt Translation Service）。
interrupt-controller@a00000: GPIO 中断控制器。
interrupt-controller@4210000: MCU 子系统的中断控制器。
interrupt-controller@48000000: TI SCI 中断控制器。
pruss@30040000/interrupt-controller@20000: PRUSS 的中断控制器。
4. 串口
serial@2800000: 主串口（serial2）。
serial@2810000: 主串口（serial3）。
serial@2b300000: 唤醒串口（wkup_uart0）。
serial@4a00000: MCU 子系统串口（mcu_uart0）。
5. I²C 总线
i2c@20000000: 主 I²C 总线 0。
i2c@20010000: 主 I²C 总线 1。
i2c@20020000: 主 I²C 总线 2。
i2c@2b200000: 唤醒 I²C 总线（wkup_i2c0）。
i2c@4900000: MCU 子系统 I²C 总线（mcu_i2c0）。
6. SPI 总线
spi@20100000: 主 SPI 总线 0。
spi@20110000: 主 SPI 总线 1。
spi@20120000: 主 SPI 总线 2。
spi@4b00000: MCU 子系统 SPI 总线 0。
spi@4b10000: MCU 子系统 SPI 总线 1。
spi@fc40000: OSPI 总线。
7. 存储设备
mmc@fa10000: eMMC 控制器。
mmc@fa00000: SD 卡控制器。
mmc@fa20000: 其他存储设备。
8. GPIO
gpio@600000: 主 GPIO 控制器 0。
gpio@601000: 主 GPIO 控制器 1。
gpio@4201000: MCU 子系统 GPIO 控制器。
9. 电源管理
fixed-regulator-vcc12v0: 固定电压调节器（12V）。
fixedregulator-vcc5v0: 固定电压调节器（5V）。
fixedregulator-vcc3v3: 固定电压调节器（3.3V）。
gpio-regulator-sd-dv: GPIO 控制的电压调节器。
net-5g-rst: 固定电压调节器，用于 5G 模块复位。
regulator-6: 固定电压调节器，用于 WLAN。
10. 时钟
clk_ov5645_fixed: 固定时钟（24 MHz）。
clk_es8388_fixed: 固定时钟（11.2896 MHz）。
bus@f0000/dmsc@44043000/clocks: 时钟控制器。
11. 定时器
timer-cl0-cpu0: ARMv8 定时器。
timer-pwm@0 至 timer-pwm@7: PWM 定时器。
pwm@23000000: PWM 控制器。
12. 音频
es8388@10: 音频编解码器。
mcasp@02B00000: 音频接口（I2S/TDM）。
sound: 简单音频卡。
13. 摄像头
camera@3c: OV5645 摄像头模块。
ticsi2rx@30102000: CSI2 接收器。
phy@30110000: DPHY 模块。
14. 触摸屏
gt928_ts@14: GT928 触摸屏控制器。
gt911_ts@5d: GT911 触摸屏控制器。
tsc2007@48: TSC2007 触摸屏控制器。
15. 网络
ethernet@8000000: CPSW3G 以太网控制器。
mdio@f00: MDIO 控制器。
ethernet-phy@1 和 ethernet-phy@2: PHY 节点。
16. 固件
psci: ARM PSCI（电源状态协调接口）。
optee: OP-TEE（可信执行环境）。
17. 其他外设
watchdog@e000000: 看门狗定时器。
crypto@40900000: 加密模块。
dss@30200000: 显示子系统。
dwc3-usb@f900000 和 dwc3-usb@f910000: USB 控制器。
mcan@20701000: CAN 控制器。
spinlock@2a000000: 硬件自旋锁。
mailbox@29000000: 邮箱模块。
pruss@30040000: PRUSS 实时子系统。
18. 引脚复用
pinctrl@f4000: 主引脚复用控制器。
pinctrl@4084000: MCU 子系统引脚复用控制器。
19. LED 和按键
leds: GPIO 控制的 LED。
keys: 按键节点。
20. 总线
bus@f0000: 主总线。
bus@4000000: MCU 子系统总线。
bus@2b000000: 唤醒总线。
bus@fc00000: OSPI 总线。

-------------

1. CPU 节点
节点: /cpus/cpu@0, /cpus/cpu@1
依赖:
psci: /firmware/psci
用于多核处理器的电源管理。
interrupt-controller@1800000: ARM GICv3 中断控制器。
2. 中断控制器
节点: /bus@f0000/interrupt-controller@1800000
依赖:
无直接依赖。
被依赖:
串口: /bus@f0000/serial@2800000, /bus@f0000/serial@2810000 等。
eMMC/SD 卡: /bus@f0000/mmc@fa10000, /bus@f0000/mmc@fa00000。
GPIO: /bus@f0000/gpio@600000, /bus@f0000/gpio@601000。
其他设备: 几乎所有设备的中断都依赖该节点。
3. 串口
节点: /bus@f0000/serial@2800000, /bus@f0000/serial@2810000
依赖:
中断控制器: /bus@f0000/interrupt-controller@1800000
电源控制器: /bus@f0000/dmsc@44043000/power-controller
时钟控制器: /bus@f0000/dmsc@44043000/clocks
引脚复用配置: /bus@f0000/pinctrl@f4000/main-uart0-pins-default, /bus@f0000/pinctrl@f4000/main-uart1-pins-default
4. eMMC/SD 卡
节点: /bus@f0000/mmc@fa10000, /bus@f0000/mmc@fa00000
依赖:
中断控制器: /bus@f0000/interrupt-controller@1800000
电源控制器: /bus@f0000/dmsc@44043000/power-controller
时钟控制器: /bus@f0000/dmsc@44043000/clocks
引脚复用配置: /bus@f0000/pinctrl@f4000/main-mmc0-pins-default, /bus@f0000/pinctrl@f4000/main-mmc1-pins-default
电压调节器:
vmmc-supply: /fixed-regulator-sd
vqmmc-supply: /gpio-regulator-sd-dv
5. GPIO
节点: /bus@f0000/gpio@600000, /bus@f0000/gpio@601000
依赖:
中断控制器: /bus@f0000/interrupt-controller@a00000
电源控制器: /bus@f0000/dmsc@44043000/power-controller
时钟控制器: /bus@f0000/dmsc@44043000/clocks
被依赖:
触摸屏控制器: /bus@f0000/i2c@20020000/gt928_ts@14, /bus@f0000/i2c@20010000/tsc2007@48
GPIO 电压调节器: /gpio-regulator-sd-dv
6. 时钟控制器
节点: /bus@f0000/dmsc@44043000/clocks
依赖:
无直接依赖。
被依赖:
串口: /bus@f0000/serial@2800000, /bus@f0000/serial@2810000
eMMC/SD 卡: /bus@f0000/mmc@fa10000, /bus@f0000/mmc@fa00000
GPIO: /bus@f0000/gpio@600000, /bus@f0000/gpio@601000
PWM: /bus@f0000/timer-pwm@0, /bus@f0000/pwm@23000000
7. 电源控制器
节点: /bus@f0000/dmsc@44043000/power-controller
依赖:
无直接依赖。
被依赖:
串口: /bus@f0000/serial@2800000, /bus@f0000/serial@2810000
eMMC/SD 卡: /bus@f0000/mmc@fa10000, /bus@f0000/mmc@fa00000
GPIO: /bus@f0000/gpio@600000, /bus@f0000/gpio@601000
PWM: /bus@f0000/timer-pwm@0, /bus@f0000/pwm@23000000
8. PWM
节点: /bus@f0000/timer-pwm@0, /bus@f0000/pwm@23000000
依赖:
时钟控制器: /bus@f0000/dmsc@44043000/clocks
电源控制器: /bus@f0000/dmsc@44043000/power-controller
9. 触摸屏控制器
节点: /bus@f0000/i2c@20020000/gt928_ts@14, /bus@f0000/i2c@20010000/tsc2007@48
依赖:
GPIO: /bus@f0000/gpio@600000, /bus@f0000/gpio@601000
I²C 总线: /bus@f0000/i2c@20020000, /bus@f0000/i2c@20010000
10. 音频编解码器
节点: /bus@f0000/i2c@20020000/es8388@10
依赖:
I²C 总线: /bus@f0000/i2c@20020000
时钟: /clk_es8388_fixed
11. 摄像头
节点: /bus@f0000/i2c@20010000/camera@3c
依赖:
I²C 总线: /bus@f0000/i2c@20010000
时钟: /clk_ov5645_fixed
12. 固定电压调节器
节点: /fixed-regulator-vcc12v0, /fixedregulator-vcc5v0, /fixedregulator-vcc3v3
依赖:
无直接依赖。
被依赖:
GPIO 电压调节器: /gpio-regulator-sd-dv
eMMC/SD 卡: /bus@f0000/mmc@fa10000, /bus@f0000/mmc@fa00000
13. GPIO 电压调节器
节点: /gpio-regulator-sd-dv
依赖:
GPIO: /bus@f0000/gpio@600000
固定电压调节器: /fixedregulator-vcc5v0
被依赖:
eMMC/SD 卡: /bus@f0000/mmc@fa00000
14. 低功耗内存
节点: /reserved-memory/lpm-memory@a1000000
依赖:
无直接依赖。
被依赖:
DMSC 节点: /bus@f0000/dmsc@44043000


------

