#ifndef HVISOR_ABI_H
#define HVISOR_ABI_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define MEM_TYPE_RAM 0

#define MEM_TYPE_IO 1

#define MEM_TYPE_VIRTIO 2

#define CONFIG_MAGIC_VERSION 6

#define CONFIG_MAX_MEMORY_REGIONS 64

#define CONFIG_MAX_INTERRUPTS 1024

#define CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD 32

#define CONFIG_NAME_MAXLEN 32

#define CONFIG_MAX_IVC_CONFIGS 2

#define CONFIG_PCI_BUS_MAXNUM 4

#define CONFIG_MAX_PCI_DEV 32

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
  uintptr_t gicd_base;
  uintptr_t gicd_size;
  uintptr_t gicc_base;
  uintptr_t gicc_size;
  uintptr_t gicc_offset;
  uintptr_t gich_base;
  uintptr_t gich_size;
  uintptr_t gicv_base;
  uintptr_t gicv_size;
} Gicv2Config;

typedef struct Gicv3Config {
  uintptr_t gicd_base;
  uintptr_t gicd_size;
  uintptr_t gicr_base;
  uintptr_t gicr_size;
  uintptr_t gits_base;
  uintptr_t gits_size;
} Gicv3Config;

enum GicConfig_Tag
#if __STDC_VERSION__ >= 202311L
  : uintptr_t
#endif // __STDC_VERSION__ >= 202311L
 {
  Gicv2,
  Gicv3,
};
#if __STDC_VERSION__ >= 202311L
typedef enum GicConfig_Tag GicConfig_Tag;
#else
typedef uintptr_t GicConfig_Tag;
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
  : uintptr_t
#endif // __STDC_VERSION__ >= 202311L
 {
  Uefi,
  NoUefi,
};
#if __STDC_VERSION__ >= 202311L
typedef enum UefiConfig_Tag UefiConfig_Tag;
#else
typedef uintptr_t UefiConfig_Tag;
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
  uintptr_t plic_base;
  uintptr_t plic_size;
  uintptr_t aplic_base;
  uintptr_t aplic_size;
} Riscv64ArchConfig;

typedef struct Loongarch64ArchConfig {
  uintptr_t dummy;
} Loongarch64ArchConfig;

typedef struct X86_64ArchConfig {
  /**
   * Base address of IOAPIC MMIO registers, usually 0xfec00000.
   */
  uintptr_t ioapic_base;
  /**
   * Size of IOAPIC MMIO registers, usually 0x1000.
   */
  uintptr_t ioapic_size;
  /**
   * Start GPA of vmlinux.bin, usually 0x100000.
   */
  uintptr_t kernel_entry_gpa;
  /**
   * GPA of the Linux boot command line.
   */
  uintptr_t cmdline_load_gpa;
  /**
   * Start GPA of setup.bin, address length no bigger than 16 bits.
   */
  uintptr_t setup_load_gpa;
  /**
   * Start GPA of initrd, or zero when unused.
   */
  uintptr_t initrd_load_gpa;
  /**
   * Size of initrd.
   */
  uintptr_t initrd_size;
  /**
   * Memory region ID where the RSDP table is copied.
   */
  uintptr_t rsdp_memory_region_id;
  /**
   * Memory region ID where other ACPI tables are copied.
   */
  uintptr_t acpi_memory_region_id;
  uintptr_t uefi_memory_region_id;
  /**
   * Start GPA of the framebuffer, or zero when unused.
   */
  uintptr_t screen_base;
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

#endif  /* HVISOR_ABI_H */
