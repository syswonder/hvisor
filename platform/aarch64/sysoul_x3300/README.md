# sysoul_x3300-dual — RK3588 dual-zone (Linux NPU zone1 + Android zone2)

Dual-zone variant of `sysoul_x3300` / `sysoul_x3300-scmi` on the same
Rockchip RK3588 "sysoul x3300" board:

| Zone | CPUs | Role | Devices |
|------|------|------|---------|
| **Zone 0 (root-linux)** | 0, 1 | hvisor.ko + hvisor-tool (VirtIO-SCMI server), console on **plain uart2** (`serial@feb50000` = ttyS2), rootfs on the **SD card** (`sdmmc@fe2c0000`), network via **gmac** passthrough | CRU/PMU/GRF, sdmmc, gmac |
| **Zone 1 (npu linux)** | 4, 5 | NPU inference (rknn), virtio console/net, rootfs on virtio-blk | RKNPU passthrough (no IOMMU, `npu-memory@38000000` 64 MB shared-dma-pool), SCMI clk 0-7 / rst 0-5 / pd 0-2 |
| **Zone 2 (android)** | 6, 7 | Android 13 GUI, virtio console/net, clocks via VirtIO-SCMI | **Passthrough**: Mali GPU, VOP2+VOP IOMMU, DSI0+dcphy0, eMMC (`mmc@fe2e0000`), gpio2 (panel), lcd backlight pwm. SCMI clk/rst/pd for all of them |

## Files

- `board.rs` — root zone layout. vs `sysoul_x3300-scmi`: eMMC (INTID 0xed),
  dsi0 (0xc7), gpio2 (0x137), lcd-pwm block (0x178/0x179) and fiq-debugger
  (0x1c7) IRQs removed from the root bitmap (moved to zone2 / unused);
  everything else identical.
- `image/dts/zone0.dts` — root dts (based on the scmi evb7-v11 root dts):
  uart2 console, SD-card rootfs, eMMC/gpio2/pwm/backlight/fiq-debugger
  disabled, `hvisor_virtio_device` extended with the zone2 resource list
  (clock indices 38-50, reset indices 30-39; indices 0-37 unchanged so the
  npu/gpu/vop/hdmi mappings of the scmi platform stay valid).
- `image/dts/zone1-linux-npu.dts` — npu zone dts, cpus 4/5 (cpu section
  spliced from the validated zone2-linux-gpu-hdmi template).
- `image/dts/zone2-android.dts` — android zone dts, generated from the
  working passthrough android dts (`sysoul_x3300` `zone1-android.dts`):
  cpus 6/7, 2 GB RAM at 0x58000000, all non-zone2 nodes disabled, enabled
  devices rewired to VirtIO-SCMI. **See the header comment in the file for
  the full list of deltas vs bare-metal android.**
- `configs/zone1-linux-npu.json`, `configs/zone2-android.json` — full zone
  configs (zone start).
- `configs/zones-npu-android.json` — virtio activation file for both zones.

## SCMI virtual id layout (zone2-android)

Virtual ids are positions in `clock_ids` / `reset_ids` / `power_ids` of
`zone2-android.json`, which index into zone0's `hvisor_virtio_device`
lists. Layout used by `zone2-android.dts`:

| virtual | resource (hvisor list idx) |
|---------|----------------------------|
| clk 0-3 | gpu clk_mali/coregroup/stacks/clk_gpu (idx 8-11) |
| clk 4-13 | vop aclk/hclk/dclk_vp0-3/pclk/dclk_src_vp0-2 (idx 12-21) |
| clk 17-18 | dsi0 pclk/sys_clk (idx 47-48) |
| clk 19-20 | dcphy0 pclk/ref (idx 49-50) |
| clk 21-25 | eMMC core/bus/axi/block/timer (idx 38-42) |
| clk 26-27 | gpio2 (idx 43-44) |
| clk 28-29 | lcd pwm / pclk (idx 45-46) |
| rst 0-5 | vop axi/ahb/dclk_vp0-3 (idx 6-11) |
| rst 6-10 | eMMC core..timer (idx 30-34) |
| rst 11 | dsi0 apb (idx 35) |
| rst 12-15 | dcphy0 m_phy/apb/grf/s_phy (idx 36-39) |
| pd 0 | gpu (idx 3 = 0x0c) |
| pd 1 | vop + dsi0 (idx 4 = 0x18) |

## Build

```bash
make all BID=aarch64/sysoul_x3300-dual
# device trees:
make -C platform/aarch64/sysoul_x3300-dual/image/dts
```

## Deploy & boot (tftp, u-boot)

Files (deploy with a `202609-` prefix next to the existing `202605-` ones):

- `hvisor.bin` (BID sysoul_x3300-dual)
- `sysoul_iron.dtb` (u-boot board dts) — unchanged from the scmi flow
- `zone0.dtb` + Linux 6.1 `Image` (root zone; rootfs partition on the SD
  card, u-boot configures the eMMC/SD layout)
- zone1: `202605-Image` (linux SDK Image) + `zone1-linux-npu.dtb` +
  `202605-rootfs.img` (virtio-blk)
- zone2: `202609-android-Image` + `202609-android-ramdisk` (extracted from
  boot.img) + `zone2-android.dtb`

```bash
setenv ipaddr 192.168.255.2; setenv netmask 255.255.255.0; setenv serverip 192.168.255.1
setenv board_dtb_addr 0x00400000; setenv hvisor_addr 0x00500000
setenv root_linux_dtb_addr 0x20000000; setenv kernel_addr 0x20400000
tftp ${board_dtb_addr} 202605-sysoul_iron.dtb; tftp ${hvisor_addr} 202609-hvisor.bin; \
tftp ${root_linux_dtb_addr} 202609-sysoul_x3300_zone0.dtb; tftp ${kernel_addr} 202605-Image; \
bootm ${hvisor_addr} - ${board_dtb_addr}
```

On root-linux:

```bash
./hvisor zone start ./zone1-linux-npu.json
./hvisor zone start ./zone2-android.json
```

(zone kernel/dtb/ramdisk paths are relative to the hvisor-tool working
directory; adjust to the deployed file names.)

## Prerequisites / known work items (outside this repo)

1. **Android kernel 5.10** (rockchip_android13_sdk): no
   `CONFIG_ARM_SCMI_TRANSPORT_VIRTIO` in 5.10 — the virtio SCMI transport
   (present in the 6.1 SDK kernel) must be backported, or the zone2 clock
   consumers will defer. `CONFIG_VIRTIO_CONSOLE=y`, `CONFIG_VIRTIO_NET=y`,
   `CONFIG_VIRTIO_MMIO=y` are already enabled.
2. **zone2 android dts source of truth**: the shipped `zone2-android.dtb`
   is self-contained; if you regenerate from the SDK, the *effective* board
   dts is `rk3588-evb7-lp4-v10-test.dts` (not `xiuos_rk3588*.dts`).
3. eMMC remains a full passthrough device: android boots with
   `androidboot.boot_devices=fe2e0000.mmc` exactly as before; no
   virtio-blk / init by-name changes are needed.
4. Android has no physical uart (console=hvc0, no earlycon): kernel early
   output before hvc0 registration is invisible by design.
5. Root zone console moved from uart3/ttyFIQ0 to plain uart2 (ttyS2) —
   both are `0xfeb50000`/`0xfeb60000` respectively; uart2 is the physical
   debug uart shared with hvisor's own early logs.

## Verification

- Zone 1: `rknpu` probed in the guest; rknn workload (e.g.
  rknn-toolkit-lite, MobileNet) at the numbers previously reached on
  sysoul_x3300-scmi.
- Zone 2: android boots to launcher on the DSI panel; console via
  `hvisor`'s virtio console (attach to the printed pts); `adb` over
  virtio-net; eMMC storage works with zero android init changes.
- Root: serial on uart2, rootfs on SD, ssh over gmac.
