// SPDX-License-Identifier: GPL-2.0-only
#ifndef __HVISOR_VIRTIO_PCI_BACKEND_H
#define __HVISOR_VIRTIO_PCI_BACKEND_H

#include "virtio_pci_bridge.h"

int hvisor_virtio_pci_get_info(struct hvisor_virtio_pci_info __user *arg);
int hvisor_virtio_pci_notify(struct virtio_pci_hypercall_info __user *arg);
int hvisor_virtio_pci_init(void);
void hvisor_virtio_pci_exit(void);

#endif /* __HVISOR_VIRTIO_PCI_BACKEND_H */
