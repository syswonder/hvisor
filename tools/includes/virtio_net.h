#ifndef _HVISOR_VIRTIO_NET_H
#define _HVISOR_VIRTIO_NET_H
#include "virtio.h"
#include "event_monitor.h"

// Queue idx for virtio net.
#define NET_QUEUE_RX    0
#define NET_QUEUE_TX    1

// Maximum number of queues for Virtio net
#define NET_MAX_QUEUES  2

// Virtio net feature bits.
#define	VIRTIO_NET_F_MAC	(1 <<  5) /* device provides MAC */
#define	VIRTIO_NET_F_MRG_RXBUF	(1 << 15) /* driver can merge RX buffers */
#define	VIRTIO_NET_F_STATUS	(1 << 16) /* status field in virtio_net_config is available */

#define VIRTIO_NET_S_LINK_UP 1

#define VIRTQUEUE_NET_MAX_SIZE 256

struct virtio_net_config {
    uint8_t  mac[6];
    uint16_t status;
} __attribute__((packed));

struct virtio_net_hdr {
    uint8_t		flags;
    uint8_t		gso_type;
    uint16_t	hdr_len;
    uint16_t	gso_size;
    uint16_t	csum_start;
    uint16_t	csum_offset;
    uint16_t	num_buffers;
} __attribute__((packed));

typedef struct virtio_net_config NetConfig;
typedef struct virtio_net_hdr NetHdr;

typedef struct virtio_net_dev {
    NetConfig config;
    int tapfd;
    int rx_ready;   // If rxq has available empty buffers.
    struct hvisor_event *event;
} NetDev;

NetDev *init_net_dev(uint8_t mac[]);

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

void virtio_net_rx_callback(int fd, int epoll_type, void *param);
int virtio_net_init(VirtIODevice *vdev, char *devname);
#endif //_HVISOR_VIRTIO_NET_H
