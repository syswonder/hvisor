#ifndef _HVISOR_VIRTIO_NET_H
#define _HVISOR_VIRTIO_NET_H
#include "virtio.h"
#include "mevent.h"
/*
 * Queue definitions.
 */
#define VIRTIO_NET_RXQ	0
#define VIRTIO_NET_TXQ	1
//#define VIRTIO_NET_CTLQ	2	/* not yet supported */

#define VIRTIO_NET_MAXQ	2

/*
 * Host capabilities.  Note that we only offer a few of these.
 */
#define	VIRTIO_NET_F_CSUM	(1 <<  0) /* host handles partial cksum */
#define	VIRTIO_NET_F_GUEST_CSUM	(1 <<  1) /* guest handles partial cksum */
#define	VIRTIO_NET_F_MAC	(1 <<  5) /* host supplies MAC */
#define	VIRTIO_NET_F_GSO_DEPREC	(1 <<  6) /* deprecated: host handles GSO */
#define	VIRTIO_NET_F_GUEST_TSO4	(1 <<  7) /* guest can rcv TSOv4 */
#define	VIRTIO_NET_F_GUEST_TSO6	(1 <<  8) /* guest can rcv TSOv6 */
#define	VIRTIO_NET_F_GUEST_ECN	(1 <<  9) /* guest can rcv TSO with ECN */
#define	VIRTIO_NET_F_GUEST_UFO	(1 << 10) /* guest can rcv UFO */
#define	VIRTIO_NET_F_HOST_TSO4	(1 << 11) /* host can rcv TSOv4 */
#define	VIRTIO_NET_F_HOST_TSO6	(1 << 12) /* host can rcv TSOv6 */
#define	VIRTIO_NET_F_HOST_ECN	(1 << 13) /* host can rcv TSO with ECN */
#define	VIRTIO_NET_F_HOST_UFO	(1 << 14) /* host can rcv UFO */
#define	VIRTIO_NET_F_MRG_RXBUF	(1 << 15) /* host can merge RX buffers */
#define	VIRTIO_NET_F_STATUS	(1 << 16) /* config status field available */
#define	VIRTIO_NET_F_CTRL_VQ	(1 << 17) /* control channel available */
#define	VIRTIO_NET_F_CTRL_RX	(1 << 18) /* control channel RX mode support */
#define	VIRTIO_NET_F_CTRL_VLAN	(1 << 19) /* control channel VLAN filtering */
#define	VIRTIO_NET_F_GUEST_ANNOUNCE \
				(1 << 21) /* guest can send gratuitous pkts */

#define VIRTIO_NET_FEATURES \
    ( VIRTIO_F_VERSION_1 | VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS | VIRTIO_NET_F_MRG_RXBUF)

#define VIRTIO_NET_S_LINK_UP 1

#define VIRTQUEUE_NET_MAX_SIZE 256
/*
 * MMIO config-space "registers"
 */
struct virtio_net_config {
    uint8_t  mac[6];
    uint16_t status;
} __attribute__((packed));

struct virtio_net_rxhdr {
    uint8_t		vrh_flags;
    uint8_t		vrh_gso_type;
    uint16_t	vrh_hdr_len;
    uint16_t	vrh_gso_size;
    uint16_t	vrh_csum_start;
    uint16_t	vrh_csum_offset;
    uint16_t	vrh_bufs;
} __attribute__((packed));

typedef struct virtio_net_config NetConfig;
typedef struct virtio_net_rxhdr NetRxHdr;

typedef struct virtio_net_dev {
    NetConfig config;
    int tapfd;
    int rx_ready;   // If rxq has available empty buffers.
    int rx_vhdrlen; // rx buf header length
    int rx_merge;   // In default, VIRTIO_NET_F_MRG_RXBUF feature will be enabled, and rx_merge is 1.
    struct mevent *mevp;
} NetDev;

NetDev *init_net_dev(uint8_t mac[]);

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

int virtio_net_init(VirtIODevice *vdev, char *devname);
#endif //_HVISOR_VIRTIO_NET_H
