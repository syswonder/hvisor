// SPDX-License-Identifier: GPL-2.0-only
#ifndef __VIRTIO_PCI_USERSPACE_H__
#define __VIRTIO_PCI_USERSPACE_H__

#include <linux/virtio_blk.h>
#include <linux/virtio_ring.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "virtio_pci_bridge.h"

typedef struct {
    void *base;
    size_t len;
} MappedRegion;

typedef struct {
    volatile struct virtq_desc *desc;
    volatile struct virtq_avail *avail;
    volatile struct virtq_used *used;
    MappedRegion desc_region;
    MappedRegion avail_region;
    MappedRegion used_region;
    __u16 queue_size;
    int zone_id;
} UserspacePciVirtQueue;

typedef struct {
    int image_fd;
    uint64_t capacity_sectors;
    char id[VIRTIO_BLK_ID_BYTES];
} UserspacePciBlkDev;

typedef struct {
    bool active;
    __u16 dtype;
    __u16 num_of_vq;
    UserspacePciVirtQueue vqs[MAX_VQ];
    union {
        UserspacePciBlkDev blk;
    } backend;
} UserspacePciDev;

uint32_t userspace_pci_le32_to_cpu(uint32_t value);
uint64_t userspace_pci_le64_to_cpu(uint64_t value);
size_t userspace_pci_virtio_avail_ring_size(__u64 queue_size);
size_t userspace_pci_virtio_used_ring_size(__u64 queue_size);
int userspace_pci_map_phys_region(__u64 phys, size_t len, MappedRegion *region,
                                  void **mapped_ptr);
void userspace_pci_unmap_region(MappedRegion *region);
int userspace_pci_translate_addr(int zone_id, __u64 ipa, size_t len,
                                 void **mapped_ptr);

int userspace_pci_rng_init_dev(UserspacePciDev *dev,
                               const struct virtio_pci_config_info *info);
void userspace_pci_rng_cleanup_dev(UserspacePciDev *dev);
int userspace_pci_rng_process_vq(UserspacePciVirtQueue *vq);

int userspace_pci_blk_init_dev(UserspacePciDev *dev,
                               const struct virtio_pci_config_info *info);
void userspace_pci_blk_cleanup_dev(UserspacePciDev *dev);
int userspace_pci_blk_process_vq(UserspacePciDev *dev,
                                 UserspacePciVirtQueue *vq);

int virtio_pci_userspace_backend_init(void);
void virtio_pci_userspace_backend_shutdown(void);

#endif
