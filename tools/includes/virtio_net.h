#ifndef _HVISOR_VIRTIO_NET_H
#define _HVISOR_VIRTIO_NET_H
#include "virtio.h"
#include <linux/virtio_net.h>
#include "event_monitor.h"

// Queue idx for virtio net.
#define NET_QUEUE_RX    0
#define NET_QUEUE_TX    1

// Maximum number of queues for Virtio net
#define NET_MAX_QUEUES  2

#define VIRTQUEUE_NET_MAX_SIZE 256

#define NET_SUPPORTED_FEATURES ( (1ULL << VIRTIO_F_VERSION_1) | (1ULL << VIRTIO_NET_F_MAC) | (1ULL << VIRTIO_NET_F_STATUS) )

typedef struct virtio_net_config NetConfig;
typedef struct virtio_net_hdr_v1 NetHdr;

typedef struct virtio_net_dev {
    NetConfig config;
    int tapfd;
    int rx_ready;   
    struct hvisor_event *event;
} NetDev;

NetDev *init_net_dev(uint8_t mac[]);

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

void virtio_net_rx_callback(int fd, int epoll_type, void *param);
int virtio_net_init(VirtIODevice *vdev, char *devname);
#endif //_HVISOR_VIRTIO_NET_H
