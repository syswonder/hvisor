// SPDX-License-Identifier: GPL-2.0-only
#include <inttypes.h>
#include <string.h>
#include <sys/mman.h>

#include "log.h"
#include "virtio_internal.h"

pthread_mutex_t ZONE_MEM_MUTEX = PTHREAD_MUTEX_INITIALIZER;
struct zone_mem zone_mem[MAX_ZONES];

void *get_virt_addr(void *zonex_ipa, int zone_id) {
    if (zone_id < 0 || zone_id >= MAX_ZONES)
        return NULL;

    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    struct zone_mem *z = &zone_mem[zone_id];
    uintptr_t ipa = (uintptr_t)zonex_ipa;

    for (size_t i = 0; i < z->num_regions; i++) {
        uintptr_t lef = z->regions[i].zonex_ipa;
        uintptr_t rig = z->regions[i].zonex_ipa + z->regions[i].mem_size;
        if (lef <= ipa && ipa < rig) {
            void *virt_addr = (void *)(ipa - lef + z->regions[i].virt_addr);
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            return virt_addr;
        }
    }

    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    log_error("can't find zone mem index for zonex_ipa = 0x%" PRIxPTR, ipa);
    return NULL;
}

static uint64_t range_end(uint64_t base, uint64_t len) {
    return UINT64_MAX - base < len ? UINT64_MAX : base + len;
}

static bool ranges_overlap(uint64_t base_a, uint64_t len_a, uint64_t base_b,
                           uint64_t len_b) {
    if (len_a == 0 || len_b == 0)
        return false;
    return base_a < range_end(base_b, len_b) &&
           base_b < range_end(base_a, len_a);
}

static bool same_memory_region(const VirtioMemoryRegionConfig *a,
                               const VirtioMemoryRegionConfig *b) {
    return a->zone0_ipa == b->zone0_ipa && a->zonex_ipa == b->zonex_ipa &&
           a->size == b->size;
}

static int find_zone_memory_region_locked(uint32_t zone_id,
                                          const VirtioMemoryRegionConfig *mem) {
    struct zone_mem *z = &zone_mem[zone_id];
    for (size_t i = 0; i < z->num_regions; i++) {
        struct zone_mem_region *existing = &z->regions[i];
        VirtioMemoryRegionConfig current = {
            .zone0_ipa = existing->zone0_ipa,
            .zonex_ipa = existing->zonex_ipa,
            .size = existing->mem_size,
        };
        if (same_memory_region(&current, mem))
            return (int)i;
        if (ranges_overlap(mem->zone0_ipa, mem->size, current.zone0_ipa,
                           current.size) ||
            ranges_overlap(mem->zonex_ipa, mem->size, current.zonex_ipa,
                           current.size))
            return -2;
    }
    return -1;
}

int virtio_mmap_zone_memory(const VirtioZoneConfig *cfg) {
    for (size_t i = 0; i < cfg->memory_region_num; i++) {
        const VirtioMemoryRegionConfig *mem = &cfg->memory_regions[i];
        pthread_mutex_lock(&ZONE_MEM_MUTEX);
        int found = find_zone_memory_region_locked(cfg->zone_id, mem);
        if (found >= 0) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            continue;
        }
        if (found == -2 || mem->size == 0 ||
            zone_mem[cfg->zone_id].num_regions >= CONFIG_MAX_MEMORY_REGIONS) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            return -1;
        }
        size_t slot = zone_mem[cfg->zone_id].num_regions;
        pthread_mutex_unlock(&ZONE_MEM_MUTEX);

        void *virt_addr = mmap(NULL, mem->size, PROT_READ | PROT_WRITE,
                               MAP_SHARED, ko_fd, (off_t)mem->zone0_ipa);
        if (virt_addr == MAP_FAILED)
            return -1;

        pthread_mutex_lock(&ZONE_MEM_MUTEX);
        if (slot != zone_mem[cfg->zone_id].num_regions) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            munmap(virt_addr, mem->size);
            return -1;
        }
        zone_mem[cfg->zone_id].regions[slot] = (struct zone_mem_region){
            .virt_addr = (uintptr_t)virt_addr,
            .zone0_ipa = mem->zone0_ipa,
            .zonex_ipa = mem->zonex_ipa,
            .mem_size = mem->size,
        };
        zone_mem[cfg->zone_id].num_regions++;
        pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    }
    return 0;
}

void virtio_unmap_all_memory(void) {
    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    for (int i = 0; i < MAX_ZONES; i++) {
        struct zone_mem *z = &zone_mem[i];
        for (size_t j = 0; j < z->num_regions; j++)
            if (z->regions[j].mem_size != 0)
                munmap((void *)z->regions[j].virt_addr, z->regions[j].mem_size);
        memset(z, 0, sizeof(*z));
    }
    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
}
