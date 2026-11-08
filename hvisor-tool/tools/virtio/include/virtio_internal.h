/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef HVISOR_VIRTIO_INTERNAL_H
#define HVISOR_VIRTIO_INTERNAL_H

#include "safe_cjson.h"
#include "virtio.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <sys/types.h>

struct zone_mem_region {
    uintptr_t virt_addr;
    uintptr_t zone0_ipa;
    uintptr_t zonex_ipa;
    uintptr_t mem_size;
};

struct zone_mem {
    struct zone_mem_region regions[CONFIG_MAX_MEMORY_REGIONS];
    size_t num_regions;
};

typedef struct virtio_memory_region_config {
    uint64_t zone0_ipa;
    uint64_t zonex_ipa;
    uint64_t size;
} VirtioMemoryRegionConfig;

typedef struct virtio_device_config {
    VirtioDeviceType type;
    uint32_t zone_id;
    uint64_t addr;
    uint64_t len;
    uint32_t irq;
    bool enabled;
    const struct virtio_config_ops *config_ops;
    VirtioDeviceParams params;
} VirtioDeviceConfig;

typedef struct virtio_zone_config {
    uint32_t zone_id;
    size_t memory_region_num;
    VirtioMemoryRegionConfig memory_regions[CONFIG_MAX_MEMORY_REGIONS];
    size_t device_num;
    VirtioDeviceConfig devices[MAX_DEVS];
} VirtioZoneConfig;

typedef struct virtio_config {
    size_t zone_num;
    VirtioZoneConfig zones[MAX_ZONES];
} VirtioConfig;

extern volatile struct virtio_bridge *virtio_bridge;
extern int ko_fd;
extern VirtIODevice *vdevs[MAX_DEVS];
extern int vdevs_num;
extern pthread_mutex_t VDEV_MUTEX;
extern pthread_mutex_t ZONE_MEM_MUTEX;
extern struct zone_mem zone_mem[MAX_ZONES];

const struct virtio_device_ops *virtio_lookup_ops(VirtioDeviceType type);
const struct virtio_config_ops *virtio_lookup_config_ops(VirtioDeviceType type);
VirtIODevice *create_virtio_device_unpublished(
    VirtioDeviceType dev_type, uint32_t zone_id, uint64_t base_addr,
    uint64_t len, uint32_t irq_id, const VirtioDeviceParams *params);
int publish_virtio_devices(VirtIODevice **new_devs, size_t count);
void destroy_unpublished_virtio_device(VirtIODevice *vdev);
void virtio_destroy_all_devices(void);

int virtio_validate_config_devices(const VirtioConfig *cfg);
int virtio_mmap_zone_memory(const VirtioZoneConfig *cfg);
void virtio_unmap_all_memory(void);

void virtio_publish_mmio_addrs(void);
int virtio_add_from_json(const char *json_path);
void stop_virtio_control_server(void);

#endif
