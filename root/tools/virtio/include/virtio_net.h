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
} NetDev;

NetDev *init_net_dev(uint8_t mac[]);

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

void virtio_net_event_handler(int fd, int epoll_type, void *param);
int virtio_net_init(VirtIODevice *vdev, char *devname);
void virtio_net_close(VirtIODevice *vdev);
#endif //_HVISOR_VIRTIO_NET_H
