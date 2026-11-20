#ifndef HVISOR_ABI_H
#define HVISOR_ABI_H

#pragma once

#if defined(__KERNEL__)
#include <linux/types.h>
#else
#include <stdbool.h>
#include <stdint.h>
#endif

#define MEM_TYPE_RAM 0

#define MEM_TYPE_IO 1

#define MEM_TYPE_VIRTIO 2

#define CONFIG_MAGIC_VERSION 7

#define CONFIG_MAX_MEMORY_REGIONS 64

#define CONFIG_MAX_INTERRUPTS 1024

#define CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD 32

#define CONFIG_NAME_MAXLEN 32

#define CONFIG_MAX_ZONES 32

#define CONFIG_MAX_IVC_CONFIGS 2

#define CONFIG_PCI_BUS_MAXNUM 4

#define CONFIG_MAX_PCI_DEV 32

#define HVISOR_INIT_VIRTIO 256

#define HVISOR_GET_TASK 257

#define HVISOR_FINISH_REQ 258

#define HVISOR_ZONE_START 1074266371

#define HVISOR_ZONE_SHUTDOWN 1074266372

#define HVISOR_ZONE_LIST 2148008197

#define HVISOR_CONFIG_CHECK 2148008198

#define HVISOR_SET_EVENTFD 1074004231

#define HVISOR_LOAD_IMAGE 1075314952

#define HVISOR_SET_BOOT_MODE 1074266377

#define HVISOR_GET_ECAM_BASE 2148008202

#define HVISOR_DEASSERT_IRQ 1074266379

#define HVISOR_IVC_USER_INFO 2148026624

#define HVISOR_SCMI_CLOCK_GET_COUNT 1

#define HVISOR_SCMI_CLOCK_GET_ATTRIBUTES 2

#define HVISOR_SCMI_CLOCK_DESCRIBE_RATES 3

#define HVISOR_SCMI_CLOCK_RATE_GET 4

#define HVISOR_SCMI_CLOCK_RATE_SET 5

#define HVISOR_SCMI_CLOCK_CONFIG_GET 6

#define HVISOR_SCMI_CLOCK_CONFIG_SET 7

#define HVISOR_SCMI_CLOCK_NAME_GET 8

#define HVISOR_SCMI_RESET_GET_COUNT 1

#define HVISOR_SCMI_RESET_RESET 2

#define HVISOR_SCMI_POWER_GET_COUNT 1

#define HVISOR_SCMI_POWER_GET_ATTRIBUTES 2

#define HVISOR_SCMI_POWER_STATE_SET 3

#define HVISOR_SCMI_POWER_STATE_GET 4

#define SCMI_POWER_STATE_GENERIC_ON 0

#define SCMI_POWER_STATE_GENERIC_OFF 1073741824

#define HVISOR_SCMI_CLOCK_IOCTL 3226992928

#define HVISOR_SCMI_RESET_IOCTL 3222536481

#define HVISOR_SCMI_POWER_IOCTL 3226468642

#define MAX_REQ 32

#define MAX_DEVS 8

#define MAX_CPUS 32

#define MAX_VQ 16

#define VIRTIO_PCI_HYPERCALL_VERSION 1

#define MAX_PCI_CONFIG_REQ 4

#define MAX_PCI_CONFIG_RES 4

#define MAX_PCI_DATA_REQ 32

enum HyperCallCode
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  HvVirtioInit = 0,
  HvVirtioInjectIrq = 1,
  HvVirtioGetIrq = 86,
  HvZoneStart = 2,
  HvZoneShutdown = 3,
  HvZoneList = 4,
  HvClearInjectIrq = 11,
  HvIvcInfo = 5,
  HvConfigCheck = 6,
  HvVirtioPCI = 7,
  HvZoneSetBootMode = 8,
  HvGetEcamBase = 9,
};
#if __STDC_VERSION__ >= 202311L
typedef enum HyperCallCode HyperCallCode;
#else
typedef uint64_t HyperCallCode;
#endif // __STDC_VERSION__ >= 202311L

typedef enum VpciDevType {
  Physical = 0,
  StandardVdev = 1,
  VirtioRng = 2,
  VirtioBlk = 3,
} VpciDevType;

typedef struct HvConfigMemoryRegion {
  uint32_t mem_type;
  uint64_t physical_start;
  uint64_t virtual_start;
  uint64_t size;
} HvConfigMemoryRegion;

typedef struct IvcInfo {
  uint64_t len;
  uint64_t ivc_ct_ipas[CONFIG_MAX_IVC_CONFIGS];
  uint64_t ivc_shmem_ipas[CONFIG_MAX_IVC_CONFIGS];
  uint32_t ivc_ids[CONFIG_MAX_IVC_CONFIGS];
  uint32_t ivc_irqs[CONFIG_MAX_IVC_CONFIGS];
} IvcInfo;

typedef struct HvIvcControlTable {
  uint32_t ivc_id;
  uint32_t max_peers;
  uint32_t rw_sec_size;
  uint32_t out_sec_size;
  uint32_t peer_id;
  uint32_t ipi_invoke;
} HvIvcControlTable;

typedef struct HvIvcUserInfo {
  int32_t len;
  int32_t ivc_ids[2];
} HvIvcUserInfo;

typedef struct HvIrqLineArgs {
  uint32_t zone_id;
  uint32_t irq_id;
} HvIrqLineArgs;

typedef struct HvLoadImageArgs {
  uint64_t user_buffer;
  uint64_t size;
  uint64_t load_paddr;
} HvLoadImageArgs;

typedef struct ZoneInfo {
  uint32_t zone_id;
  uint64_t cpus;
  uint8_t name[32];
  uint8_t is_err;
} ZoneInfo;

typedef struct HvZoneListArgs {
  uint64_t cnt;
  struct ZoneInfo *zones;
} HvZoneListArgs;

typedef struct HvPciConfig {
  uint64_t ecam_base;
  uint64_t ecam_size;
  uint64_t io_base;
  uint64_t io_size;
  uint64_t pci_io_base;
  uint64_t mem32_base;
  uint64_t mem32_size;
  uint64_t pci_mem32_base;
  uint64_t mem64_base;
  uint64_t mem64_size;
  uint64_t pci_mem64_base;
  uint32_t bus_range_begin;
  uint32_t bus_range_end;
  uint8_t domain;
} HvPciConfig;

typedef struct HvIvcConfig {
  uint32_t ivc_id;
  uint32_t peer_id;
  uint64_t control_table_ipa;
  uint64_t shared_mem_ipa;
  uint32_t rw_sec_size;
  uint32_t out_sec_size;
  uint32_t interrupt_num;
  uint32_t max_peers;
} HvIvcConfig;

typedef struct HvZoneBootMode {
  uint32_t zone_id;
  uint32_t multiboot_enabled;
  uint64_t multiboot_info_paddr;
} HvZoneBootMode;

typedef struct HvPciDevConfig {
  uint8_t domain;
  uint8_t bus;
  uint8_t device;
  uint8_t function;
  uint8_t v_bus;
  uint8_t v_device;
  uint8_t v_function;
  enum VpciDevType dev_type;
} HvPciDevConfig;

typedef uint32_t BitmapWord;

/**
 * Architecture-independent part of the zone configuration ABI.
 *
 * Architecture-specific data lives in the concrete ZoneConfig wrapper, not in
 * this structure.
 */
typedef struct HvZoneConfigCommon {
  uint32_t zone_id;
  uint64_t cpus;
  uint32_t num_memory_regions;
  struct HvConfigMemoryRegion memory_regions[64];
  BitmapWord interrupts_bitmap[32];
  uint32_t num_ivc_configs;
  struct HvIvcConfig ivc_configs[2];
  uint64_t entry_point;
  uint64_t kernel_load_paddr;
  uint64_t kernel_size;
  uint64_t dtb_load_paddr;
  uint64_t dtb_size;
  uint8_t name[32];
  uint64_t num_pci_bus;
  struct HvPciConfig pci_config[4];
  uint64_t num_pci_devs;
  struct HvPciDevConfig alloc_pci_devs[32];
} HvZoneConfigCommon;

typedef struct Gicv2Config {
  uint64_t gicd_base;
  uint64_t gicd_size;
  uint64_t gicc_base;
  uint64_t gicc_size;
  uint64_t gicc_offset;
  uint64_t gich_base;
  uint64_t gich_size;
  uint64_t gicv_base;
  uint64_t gicv_size;
} Gicv2Config;

typedef struct Gicv3Config {
  uint64_t gicd_base;
  uint64_t gicd_size;
  uint64_t gicr_base;
  uint64_t gicr_size;
  uint64_t gits_base;
  uint64_t gits_size;
} Gicv3Config;

enum GicConfig_Tag
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  Gicv2,
  Gicv3,
};
#if __STDC_VERSION__ >= 202311L
typedef enum GicConfig_Tag GicConfig_Tag;
#else
typedef uint64_t GicConfig_Tag;
#endif // __STDC_VERSION__ >= 202311L

typedef struct GicConfig {
  GicConfig_Tag tag;
  union {
    struct {
      struct Gicv2Config gicv2;
    };
    struct {
      struct Gicv3Config gicv3;
    };
  };
} GicConfig;

typedef struct HvUefiConfig {
  uint64_t memory_map_addr;
  uint64_t memory_map_size;
  uint64_t sys_map_addr;
} HvUefiConfig;

enum UefiConfig_Tag
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  Uefi,
  NoUefi,
};
#if __STDC_VERSION__ >= 202311L
typedef enum UefiConfig_Tag UefiConfig_Tag;
#else
typedef uint64_t UefiConfig_Tag;
#endif // __STDC_VERSION__ >= 202311L

typedef struct UefiConfig {
  UefiConfig_Tag tag;
  union {
    struct {
      struct HvUefiConfig uefi;
    };
  };
} UefiConfig;

typedef struct Aarch64ArchConfig {
  uint8_t is_aarch32;
  struct GicConfig gic_config;
  struct UefiConfig uefi_config;
} Aarch64ArchConfig;

typedef struct Riscv64ArchConfig {
  uint64_t plic_base;
  uint64_t plic_size;
  uint64_t aplic_base;
  uint64_t aplic_size;
} Riscv64ArchConfig;

typedef struct Loongarch64ArchConfig {
  uint64_t dummy;
} Loongarch64ArchConfig;

typedef struct X86_64ArchConfig {
  /**
   * Base address of IOAPIC MMIO registers, usually 0xfec00000.
   */
  uint64_t ioapic_base;
  /**
   * Size of IOAPIC MMIO registers, usually 0x1000.
   */
  uint64_t ioapic_size;
  /**
   * Start GPA of vmlinux.bin, usually 0x100000.
   */
  uint64_t kernel_entry_gpa;
  /**
   * GPA of the Linux boot command line.
   */
  uint64_t cmdline_load_gpa;
  /**
   * Start GPA of setup.bin, address length no bigger than 16 bits.
   */
  uint64_t setup_load_gpa;
  /**
   * Start GPA of initrd, or zero when unused.
   */
  uint64_t initrd_load_gpa;
  /**
   * Size of initrd.
   */
  uint64_t initrd_size;
  /**
   * Memory region ID where the RSDP table is copied.
   */
  uint64_t rsdp_memory_region_id;
  /**
   * Memory region ID where other ACPI tables are copied.
   */
  uint64_t acpi_memory_region_id;
  uint64_t uefi_memory_region_id;
  /**
   * Start GPA of the framebuffer, or zero when unused.
   */
  uint64_t screen_base;
} X86_64ArchConfig;

typedef struct Aarch64ZoneConfig {
  struct HvZoneConfigCommon common;
  struct Aarch64ArchConfig arch_config;
} Aarch64ZoneConfig;

typedef struct Riscv64ZoneConfig {
  struct HvZoneConfigCommon common;
  struct Riscv64ArchConfig arch_config;
} Riscv64ZoneConfig;

typedef struct Loongarch64ZoneConfig {
  struct HvZoneConfigCommon common;
  struct Loongarch64ArchConfig arch_config;
} Loongarch64ZoneConfig;

typedef struct X86_64ZoneConfig {
  struct HvZoneConfigCommon common;
  struct X86_64ArchConfig arch_config;
} X86_64ZoneConfig;

typedef struct HvisorDeviceReq {
  uint64_t src_cpu;
  uint64_t address;
  uint64_t size;
  uint64_t value;
  uint32_t src_zone;
  uint8_t is_write;
  uint8_t need_interrupt;
  uint16_t _padding;
} HvisorDeviceReq;

typedef struct HvisorDeviceRes {
  uint32_t target_zone;
  uint32_t irq_id;
} HvisorDeviceRes;

typedef struct HvVirtioBridge {
  uint32_t req_front;
  uint32_t req_rear;
  uint32_t res_front;
  uint32_t res_rear;
  struct HvisorDeviceReq req_list[MAX_REQ];
  struct HvisorDeviceRes res_list[MAX_REQ];
  uint64_t cfg_flags[MAX_CPUS];
  uint64_t cfg_values[MAX_CPUS];
  uint64_t mmio_addrs[MAX_DEVS];
  uint8_t mmio_avail;
  uint8_t need_wakeup;
} HvVirtioBridge;

typedef struct HvScmiPowerAttr {
  uint32_t domain_id;
  uint32_t flags;
  uint8_t name[64];
} HvScmiPowerAttr;

typedef struct HvScmiPowerStateInfo {
  uint32_t domain_id;
  uint32_t power_state;
} HvScmiPowerStateInfo;

typedef union HvScmiPowerData {
  uint32_t power_count;
  struct HvScmiPowerAttr power_attr;
  struct HvScmiPowerStateInfo power_state_info;
  uint8_t data[0];
} HvScmiPowerData;

typedef struct HvScmiPowerArgs {
  uint32_t subcmd;
  uint32_t data_len;
  union HvScmiPowerData u;
} HvScmiPowerArgs;

typedef struct HvScmiResetInfo {
  uint32_t domain_id;
  uint32_t flags;
  uint32_t reset_state;
} HvScmiResetInfo;

typedef union HvScmiResetData {
  uint32_t reset_count;
  struct HvScmiResetInfo reset_info;
  uint8_t data[0];
} HvScmiResetData;

typedef struct HvScmiResetArgs {
  uint32_t subcmd;
  uint32_t data_len;
  union HvScmiResetData u;
} HvScmiResetArgs;

typedef struct HvScmiClockAttr {
  uint32_t clock_id;
  uint32_t enabled;
  uint32_t parent_id;
  uint8_t clock_name[64];
  uint32_t is_valid;
} HvScmiClockAttr;

typedef struct HvScmiClockRatesInfo {
  uint32_t clock_id;
  uint32_t rate_index;
  uint32_t num_rates;
  uint32_t remaining;
  uint64_t rates[8];
} HvScmiClockRatesInfo;

typedef struct HvScmiClockRateInfo {
  uint32_t clock_id;
  uint64_t rate;
} HvScmiClockRateInfo;

typedef struct HvScmiClockRateSetInfo {
  uint32_t clock_id;
  uint32_t flags;
  uint64_t rate;
} HvScmiClockRateSetInfo;

typedef struct HvScmiClockConfigInfo {
  uint32_t clock_id;
  uint32_t flags;
  uint32_t config;
  uint32_t extended_config_val;
} HvScmiClockConfigInfo;

typedef struct HvScmiClockNameInfo {
  uint32_t clock_id;
  uint8_t name[64];
} HvScmiClockNameInfo;

typedef union HvScmiClockData {
  uint32_t clock_count;
  struct HvScmiClockAttr clock_attr;
  struct HvScmiClockRatesInfo clock_rates_info;
  struct HvScmiClockRateInfo clock_rate_info;
  struct HvScmiClockRateSetInfo clock_rate_set_info;
  struct HvScmiClockConfigInfo clock_config_info;
  struct HvScmiClockNameInfo clock_name_info;
  uint8_t data[0];
} HvScmiClockData;

typedef struct HvScmiClockArgs {
  uint32_t subcmd;
  uint32_t data_len;
  union HvScmiClockData u;
} HvScmiClockArgs;

typedef struct VirtioPCIHypercallInfo {
  uint16_t version;
  uint16_t op;
  uint32_t target_cpu;
  uint32_t request_id;
  uint32_t status;
} VirtioPCIHypercallInfo;

typedef struct VirtqueueAreaInfo {
  uint64_t desc_area;
  uint64_t avail_area;
  uint64_t used_area;
  uint64_t queue_size;
} VirtqueueAreaInfo;

typedef struct VirtioPCIConfigInfo {
  uint64_t features;
  uint16_t dev_id;
  uint16_t num_of_queues;
  uint16_t dtype;
  uint16_t padding;
  struct VirtqueueAreaInfo vqs[MAX_VQ];
} VirtioPCIConfigInfo;

typedef struct VirtioPCIConfigReq {
  uint32_t request_id;
  uint32_t padding;
  struct VirtioPCIConfigInfo info;
} VirtioPCIConfigReq;

typedef struct VirtioPCIConfigRes {
  uint32_t request_id;
  uint32_t status;
  uint16_t dev_id;
  uint16_t padding;
} VirtioPCIConfigRes;

typedef struct VirtioPCIDataInfo {
  uint16_t dev_id;
  uint16_t queue_id;
  uint16_t cpu_id;
  uint16_t msix_vector_idx;
} VirtioPCIDataInfo;

typedef struct VirtioPCIDataReq {
  uint32_t request_id;
  uint32_t padding;
  struct VirtioPCIDataInfo info;
} VirtioPCIDataReq;

typedef struct HvVirtioPCIBridgeRegion {
  struct VirtioPCIHypercallInfo hypercall_info;
  uint32_t config_req_front;
  uint32_t config_req_rear;
  uint32_t config_res_front;
  uint32_t config_res_rear;
  uint32_t data_req_front;
  uint32_t data_req_rear;
  struct VirtioPCIConfigReq config_req_list[MAX_PCI_CONFIG_REQ];
  struct VirtioPCIConfigRes config_res_list[MAX_PCI_CONFIG_RES];
  struct VirtioPCIDataReq data_req_list[MAX_PCI_DATA_REQ];
} HvVirtioPCIBridgeRegion;

#endif  /* HVISOR_ABI_H */
