#ifndef __HVISOR_ZONE_CONFIG_H
#define __HVISOR_ZONE_CONFIG_H

#include "def.h"

#define MEM_TYPE_RAM     0
#define MEM_TYPE_IO      1
#define MEM_TYPE_VIRTIO  2

#define CONFIG_MAX_MEMORY_REGIONS  16
#define CONFIG_MAX_INTERRUPTS      32
#define CONFIG_MAX_ZONES           32
#define CONFIG_NAME_MAXLEN         32

// #define CONFIG_KERNEL_ARGS_MAXLEN    256

struct memory_region {
    __u32 type;
    __u64 physical_start;
    __u64 virtual_start;
    __u64 size;
};

typedef struct memory_region memory_region_t;

#ifdef ARM64
struct arch_zone_config {
    __u64 gicd_base;
    __u64 gicr_base;
    __u64 gicd_size;
    __u64 gicr_size;
};
#endif

#ifdef RISCV64
#endif

typedef struct arch_zone_config arch_zone_config_t;

struct zone_config {
    __u32 zone_id;
    __u64 cpus;
    __u32 num_memory_regions;
    memory_region_t memory_regions[CONFIG_MAX_MEMORY_REGIONS];
    __u32 num_interrupts;
    __u32 interrupts[CONFIG_MAX_INTERRUPTS];
    __u64 entry_point;
    __u64 kernel_load_paddr;
    __u64 kernel_size;
    __u64 dtb_load_paddr;
    __u64 dtb_size;
    char name[CONFIG_NAME_MAXLEN];

    arch_zone_config_t arch_config;
};

typedef struct zone_config zone_config_t;

struct zone_info {
    __u32 zone_id;
    __u64 cpus;
    char name[CONFIG_NAME_MAXLEN];
};

typedef struct zone_info zone_info_t;

#endif