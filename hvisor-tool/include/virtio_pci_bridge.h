// SPDX-License-Identifier: GPL-2.0-only
#ifndef __HVISOR_VIRTIO_PCI_BRIDGE_H
#define __HVISOR_VIRTIO_PCI_BRIDGE_H

#include <linux/ioctl.h>
#include <linux/types.h>

#define HVISOR_HC_VIRTIO_PCI 7
#define VIRTIO_PCI_BRIDGE_SIZE 4096
#define HVISOR_VIRTIO_PCI_SHUTDOWN_DRAINED 1
#define MAX_VQ 16
#define MAX_VIRTPCI_DEV 8
#define MAX_PCI_CONFIG_REQ 4
#define MAX_PCI_CONFIG_RES 4
#define MAX_PCI_DATA_REQ 32
#define VIRTIO_PCI_HYPERCALL_VERSION 2

#define VIRTIO_PCI_USERSPACE_BLK (1U << 0)
#define VIRTIO_PCI_USERSPACE_RNG (1U << 1)
#define VIRTIO_PCI_USERSPACE_DEV_BASE (MAX_VIRTPCI_DEV / 2)

enum virtio_pci_hypercall_op {
    VIRTIO_PCI_HC_OP_CONFIG_REQ_COMPLETE = 1,
    VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE = 2,
};

enum virtio_pci_hypercall_mode {
    VIRTIO_PCI_HC_INIT = 0,
    VIRTIO_PCI_HC_DOORBELL = 1,
    VIRTIO_PCI_HC_DEINIT = 2,
    VIRTIO_PCI_HC_GET_VERSION = 3,
    VIRTIO_PCI_HC_RESOLVE_RAM = 4,
    VIRTIO_PCI_HC_OWNER_ACTIVE = 5,
};

struct virtio_pci_hypercall_info {
    __u16 version;
    __u16 op;
    __u32 target_cpu;
    __u32 request_id;
    __u32 status;
};

struct hvisor_virtio_pci_info {
    __u64 pfn;
    __u32 userspace_devices;
    __u32 reserved;
};

struct virtio_pci_owner {
    __u32 zone_id;
    __u32 zone_generation;
};

struct virtio_pci_ram_request {
    __u64 ipa;
    __u64 len;
    struct virtio_pci_owner owner;
    __u64 pa;
};

#define HVISOR_GET_VIRTIO_PCI_INFO _IOR(1, 12, struct hvisor_virtio_pci_info)
#define HVISOR_VIRTIO_PCI_NOTIFY _IOW(1, 13, struct virtio_pci_hypercall_info)
#define HVISOR_VIRTIO_PCI_RESOLVE_RAM                                          \
    _IOWR(1, 14, struct virtio_pci_ram_request)

#define HVISOR_VIRTIO_PCI_BEGIN_IO _IOR(1, 15, __u32)
#define HVISOR_VIRTIO_PCI_END_IO _IO(1, 16)
#define HVISOR_VIRTIO_PCI_OWNER_ACTIVE _IOW(1, 17, struct virtio_pci_owner)

enum virtio_pci_device_type {
    VIRTIO_PCI_BLK = 3,
    VIRTIO_PCI_RNG = 4,
};

struct virtqueue_info {
    __u64 desc_area;
    __u64 avail_area;
    __u64 used_area;
    __u64 queue_size;
};

struct virtio_pci_config_info {
    __u64 features;
    __u16 zone_id;
    __u16 num_of_queues;
    __u16 dtype;
    __u16 padding;
    struct virtqueue_info vqs[MAX_VQ];
};

struct virtio_pci_config_req {
    __u32 request_id;
    __u32 zone_generation;
    struct virtio_pci_config_info info;
};

struct virtio_pci_config_res {
    __u32 request_id;
    __u32 status;
    __u16 dev_id;
    __u16 padding;
};

struct virtio_pci_data_info {
    __u16 dev_id;
    __u16 queue_id;
    __u16 cpu_id;
    __u16 msix_vector_idx;
};

struct virtio_pci_data_req {
    __u32 request_id;
    __u32 zone_generation;
    struct virtio_pci_data_info info;
};

struct virtio_pci_bridge {
    struct virtio_pci_hypercall_info hypercall_info;
    __u32 config_req_front;
    __u32 config_req_rear;
    __u32 config_res_front;
    __u32 config_res_rear;
    __u32 data_req_front;
    __u32 data_req_rear;
    struct virtio_pci_config_req config_req_list[MAX_PCI_CONFIG_REQ];
    struct virtio_pci_config_res config_res_list[MAX_PCI_CONFIG_RES];
    struct virtio_pci_data_req data_req_list[MAX_PCI_DATA_REQ];
};

struct virtq_desc {
    __u64 addr;
    __u32 len;
    __u16 flags;
    __u16 next;
};

struct virtq_avail {
    __u16 flags;
    __u16 idx;
    __u16 ring[];
};

struct virtq_used_elem {
    __u32 id;
    __u32 len;
};

struct virtq_used {
    __u16 flags;
    __u16 idx;
    struct virtq_used_elem ring[];
};

#endif
