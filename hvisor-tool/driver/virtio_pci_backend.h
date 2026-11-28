// SPDX-License-Identifier: GPL-2.0-only
#ifndef __HVISOR_VIRTIO_PCI_BACKEND_H
#define __HVISOR_VIRTIO_PCI_BACKEND_H

#include <linux/mutex.h>

#include "virtio_pci_bridge.h"

struct virtio_pci_file {
    struct mutex lock;
    bool io_active;
};

int hvisor_virtio_pci_shutdown_zone(unsigned long zone_id);
int hvisor_virtio_pci_begin_io(struct virtio_pci_file *ctx,
                               __u32 __user *epoch);
void hvisor_virtio_pci_end_io(struct virtio_pci_file *ctx);
int hvisor_virtio_pci_owner_active(struct virtio_pci_owner __user *arg);
int hvisor_virtio_pci_get_info(struct hvisor_virtio_pci_info __user *arg);
int hvisor_virtio_pci_notify(struct virtio_pci_hypercall_info __user *arg);
int hvisor_virtio_pci_resolve_ram(struct virtio_pci_ram_request __user *arg);
void *hvisor_virtio_pci_map_ram(const struct virtio_pci_owner *owner, __u64 ipa,
                                size_t dma_len, size_t access_len);
int hvisor_virtio_pci_init(void);
void hvisor_virtio_pci_exit(void);

#endif /* __HVISOR_VIRTIO_PCI_BACKEND_H */
