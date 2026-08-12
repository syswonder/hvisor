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
#ifndef _HVISOR_VIRTIO_NET_H
#define _HVISOR_VIRTIO_NET_H
#include "event_monitor.h"
#include "virtio.h"
#include <linux/virtio_net.h>

// Queue idx for virtio net.
#define NET_QUEUE_RX 0
#define NET_QUEUE_TX 1

// Maximum number of queues for Virtio net
#define NET_MAX_QUEUES 2

#define VIRTQUEUE_NET_MAX_SIZE 256

struct virtio_net_init_params {
    uint8_t mac[6];
    const char *tap;
};

// Max iov entries for a single descriptor chain.  Each descriptor in the
// chain contributes at most one iov entry, and a chain can never exceed
// the total queue size (a single descriptor's next field cannot wrap past
// vq->num due to the loop guard in process_descriptor_chain_buf).  So
// VIRTQUEUE_NET_MAX_SIZE is the tight upper bound.
#define NET_IOV_MAX VIRTQUEUE_NET_MAX_SIZE

// VIRTIO_RING_F_INDIRECT_DESC and VIRTIO_RING_F_EVENT_IDX are supported, for
// some reason we cancel them.
#define NET_SUPPORTED_FEATURES                                                 \
    ((1ULL << VIRTIO_F_VERSION_1) | (1ULL << VIRTIO_NET_F_MAC) |               \
     (1ULL << VIRTIO_NET_F_STATUS))

typedef struct virtio_net_config NetConfig;
typedef struct virtio_net_hdr_v1 NetHdr;
typedef struct virtio_net_hdr NetHdrLegacy;
typedef struct virtio_net_dev {
    NetConfig config;
    int tapfd;
    int rx_ready;
    struct hvisor_event *event;
    struct iovec *in_iov;
    struct iovec *out_iov;
} NetDev;

NetDev *init_net_dev(uint8_t mac[]);

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

void virtio_net_event_handler(int fd, int epoll_type, void *param);
int virtio_net_init(VirtIODevice *vdev, char *devname);
void virtio_net_close(VirtIODevice *vdev);
void virtio_net_reset(VirtIODevice *vdev);
void net_on_status(VirtIODevice *vdev, uint32_t status);

extern const struct virtio_device_ops virtio_net_ops;
extern const struct virtio_config_ops virtio_net_config_ops;

#endif //_HVISOR_VIRTIO_NET_H
