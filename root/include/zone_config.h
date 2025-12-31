// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Guowei Li <2401213322@stu.pku.edu.cn>
 */
#ifndef __HVISOR_ZONE_CONFIG_H
#define __HVISOR_ZONE_CONFIG_H
#include "def.h"
#include "ivc.h"

#define MEM_TYPE_RAM 0
#define MEM_TYPE_IO 1
#define MEM_TYPE_VIRTIO 2

#define CONFIG_MAX_MEMORY_REGIONS 64

typedef __u32 BitmapWord;
#define CONFIG_MAX_INTERRUPTS 1024
#define CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD 32

#define CONFIG_MAX_ZONES 32
#define CONFIG_NAME_MAXLEN 32
#define CONFIG_MAX_PCI_DEV 32
#define CONFIG_PCI_BUS_MAXNUM 4

#define IVC_PROTOCOL_USER 0x0
#define IVC_PROTOCOL_HVISOR 0x01

struct memory_region {
    __u32 type;
    __u64 physical_start;
    __u64 virtual_start;
    __u64 size;
};

typedef struct memory_region memory_region_t;

struct hv_pci_dev_config {
    __u8 domain;
    __u8 bus;
    __u8 device;
    __u8 function;
    __u32 dev_type;
};

typedef struct hv_pci_dev_config hv_pci_dev_config_t;

struct pci_config {
    __u64 ecam_base;
    __u64 ecam_size;
    __u64 io_base;
    __u64 io_size;
    __u64 pci_io_base;
    __u64 mem32_base;
    __u64 mem32_size;
    __u64 pci_mem32_base;
    __u64 mem64_base;
    __u64 mem64_size;
    __u64 pci_mem64_base;
    __u32 bus_range_begin;
    __u32 bus_range_end;
    __u8 domain;
};

typedef struct pci_config pci_config_t;

#ifdef ARM64
struct Gicv2Config {
    __u64 gicd_base;
    __u64 gicd_size;
    __u64 gicc_base;
    __u64 gicc_size;
    __u64 gicc_offset;
    __u64 gich_base;
    __u64 gich_size;
    __u64 gicv_base;
    __u64 gicv_size;
};

struct Gicv3Config {
    __u64 gicd_base;
    __u64 gicd_size;
    __u64 gicr_base;
    __u64 gicr_size;
    __u64 gits_base;
    __u64 gits_size;
};

struct Gicv2Payload {
    __u64 gic_version_tag;
    struct Gicv2Config gicv2_config;
};

struct Gicv3Payload {
    __u64 gic_version_tag;
    struct Gicv3Config gicv3_config;
};

union GicConfig {
    struct Gicv2Payload gicv2;
    struct Gicv3Payload gicv3;
};

struct arch_zone_config {
    __u8 is_aarch32;
    union GicConfig gic_config;
};
#endif

#ifdef RISCV64
struct arch_zone_config {
    __u64 plic_base;
    __u64 plic_size;
    __u64 aplic_base;
    __u64 aplic_size;
};
#endif

#ifdef LOONGARCH64
struct arch_zone_config {
    __u64 dummy;
};
#endif

#ifdef X86_64
struct arch_zone_config {
    __u64 ioapic_base;
    __u64 ioapic_size;
    __u64 kernel_entry_gpa;
    __u64 cmdline_load_gpa;
    __u64 setup_load_gpa;
    __u64 initrd_load_gpa;
    __u64 initrd_size;
    __u64 rsdp_memory_region_id;
    __u64 acpi_memory_region_id;
    __u64 screen_base;
};
#endif

typedef struct arch_zone_config arch_zone_config_t;

struct ivc_config {
    __u32 ivc_id;
    __u32 peer_id;
    __u64 control_table_ipa;
    __u64 shared_mem_ipa;
    __u32 rw_sec_size;
    __u32 out_sec_size;
    __u32 interrupt_num;
    __u32 max_peers;
};
typedef struct ivc_config ivc_config_t;

#define CONFIG_MAGIC_VERSION 0x05

// Every time you change the struct, you should also change the
// `CONFIG_MAGIC_VERSION`
struct zone_config {
    __u32 zone_id;
    __u64 cpus;
    __u32 num_memory_regions;
    memory_region_t memory_regions[CONFIG_MAX_MEMORY_REGIONS];
    BitmapWord interrupts_bitmap[CONFIG_MAX_INTERRUPTS /
                                 CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD];
    __u32 num_ivc_configs;
    ivc_config_t ivc_configs[CONFIG_MAX_IVC_CONFIGS];

    __u64 entry_point;
    __u64 kernel_load_paddr;
    __u64 kernel_size;
    __u64 dtb_load_paddr;
    __u64 dtb_size;
    char name[CONFIG_NAME_MAXLEN];

    arch_zone_config_t arch_config;
    __u64 num_pci_bus;
    pci_config_t pci_config[CONFIG_PCI_BUS_MAXNUM];
    __u64 num_pci_devs;
    hv_pci_dev_config_t alloc_pci_devs[CONFIG_MAX_PCI_DEV];
};

typedef struct zone_config zone_config_t;

struct zone_info {
    __u32 zone_id;
    __u64 cpus;
    char name[CONFIG_NAME_MAXLEN];
    __u8 is_err;
};

typedef struct zone_info zone_info_t;

#endif
