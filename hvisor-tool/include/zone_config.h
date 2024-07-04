#ifndef __HVISOR_ZONE_CONFIG_H
#define __HVISOR_ZONE_CONFIG_H

#define MEM_TYPE_RAM  0
#define MEM_TYPE_IO   1

#define CONFIG_MAX_MEMORY_REGIONS  16
#define CONFIG_MAX_INTERRUPTS      32

// #define CONFIG_KERNEL_ARGS_MAXLEN    256

typedef unsigned int u32;
typedef unsigned long long u64;

struct memory_region {
    u32 type;
    u64 physical_start;
    u64 virtual_start;
    u64 size;
};

typedef struct memory_region memory_region_t;
       
struct zone_config {
    u32 zone_id;
    u64 cpus;
    u32 num_memory_regions;
    memory_region_t memory_regions[CONFIG_MAX_MEMORY_REGIONS];
    u32 num_interrupts;
    u32 interrupts[CONFIG_MAX_INTERRUPTS];
    u64 entry_point;
    u64 dtb_load_paddr;
};

typedef struct zone_config zone_config_t;

#endif