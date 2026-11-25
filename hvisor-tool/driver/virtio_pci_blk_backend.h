// SPDX-License-Identifier: GPL-2.0-only
#ifndef __HVISOR_VIRTIO_PCI_BLK_BACKEND_H
#define __HVISOR_VIRTIO_PCI_BLK_BACKEND_H

#include "virtio_pci_bridge.h"

int hvisor_virtio_pci_blk_init_dev(__u16 dev_id);
void hvisor_virtio_pci_blk_exit(void);
void hvisor_virtio_pci_blk_handler(__u16 dev_id, struct virtqueue_info *vq,
                                   int queue_id);

#endif /* __HVISOR_VIRTIO_PCI_BLK_BACKEND_H */
