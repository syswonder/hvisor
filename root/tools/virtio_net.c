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
#include "virtio_net.h"
#include "event_monitor.h"
#include "log.h"
#include "virtio.h"
#include <errno.h>
#include <fcntl.h>
#include <linux/if_tun.h>
#include <net/if.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/uio.h>
#include <unistd.h>
// The max bytes of a packet in data link layer is 1518 bytes.
static uint8_t trashbuf[1600];

NetDev *init_net_dev(uint8_t mac[]) {
    NetDev *dev = malloc(sizeof(NetDev));
    dev->config.mac[0] = mac[0];
    dev->config.mac[1] = mac[1];
    dev->config.mac[2] = mac[2];
    dev->config.mac[3] = mac[3];
    dev->config.mac[4] = mac[4];
    dev->config.mac[5] = mac[5];
    dev->config.status = VIRTIO_NET_S_LINK_UP;
    dev->tapfd = -1;
    dev->rx_ready = 0;
    dev->event = NULL;
    return dev;
}

// open tap device
static int open_tap(char *devname) {
    log_info("virtio net tap open");
    int tunfd;
    struct ifreq ifr;
    tunfd = open("/dev/net/tun", O_RDWR);
    if (tunfd < 0) {
        log_error("Failed to open tap device");
        return -1;
    }
    memset(&ifr, 0, sizeof(ifr));
    // IFF_NO_PI tells kernel do not provide message header
    ifr.ifr_flags = IFF_TAP | IFF_NO_PI;
    strncpy(ifr.ifr_name, devname, IFNAMSIZ);
    ifr.ifr_name[IFNAMSIZ - 1] = '\0';
    if (ioctl(tunfd, TUNSETIFF, (void *)&ifr) < 0) {
        log_error("open of tap device %s fail", devname);
        close(tunfd);
        return -1;
    }
    log_info("open virtio net tap succeed");
    return tunfd;
}

/// When driver notifies rxq, it means the rx process can now begin
int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("virtio_net_rxq_notify_handler");
    NetDev *net = vdev->dev;
    if (net->rx_ready <= 0) {
        net->rx_ready = 1;
        // When buffers are all used, virtio_net_event_handler will notify the
        // driver.
        virtqueue_disable_notify(vq);
    }
    return 0;
}
/// remove the header in iov, return the new iov. the new iov num is in niov.
static inline struct iovec *rm_iov_header(struct iovec *iov, int *niov,
                                          int header_len) {
    if (iov == NULL || *niov == 0 || iov[0].iov_len < (size_t)header_len) {
        log_error("invalid iov");
        return NULL;
    }

    iov[0].iov_len -= header_len;
    if (iov[0].iov_len > 0) {
        iov[0].iov_base = (char *)iov[0].iov_base + header_len;
        return iov;
    } else {
        *niov = *niov - 1;
        if (*niov == 0)
            return NULL;
        return iov + 1;
    }
}

size_t get_nethdr_size(VirtIODevice *vdev) {
    // Virtio 1.0 specifies the header as NetHdr. But the legacy version
    // specifies the headr as NetHdrLegacy
    if (vdev->regs.drv_feature & (1ULL << VIRTIO_F_VERSION_1)) {
        return sizeof(NetHdr);
    } else {
        return sizeof(NetHdrLegacy);
    }
}

/// Called when tap device received packets
void virtio_net_event_handler(int fd, int epoll_type, void *param) {
    log_debug("virtio_net_event_handler");
    VirtIODevice *vdev = param;
    void *vnet_header;
    struct iovec *iov, *iov_packet;
    NetDev *net = vdev->dev;
    VirtQueue *vq = &vdev->vqs[NET_QUEUE_RX];
    int n, len;
    uint16_t idx;
    size_t header_len = get_nethdr_size(vdev);
    if (fd != net->tapfd || epoll_type != EPOLLIN) {
        log_error("invalid event");
        return;
    }
    if (net->tapfd == -1 || vdev->type != VirtioTNet) {
        log_error("net rx callback should not be called");
        return;
    }

    // if vq is not setup, drop the packet
    if (!net->rx_ready) {
        read(net->tapfd, trashbuf, sizeof(trashbuf));
        return;
    }
    // if rx_vq is empty, drop the packet
    if (virtqueue_is_empty(vq)) {
        read(net->tapfd, trashbuf, sizeof(trashbuf));
        virtio_inject_irq(vq);
        return;
    }
    while (!virtqueue_is_empty(vq)) {
        n = process_descriptor_chain(vq, &idx, &iov, NULL, 0, false);
        if (n < 1 || n > VIRTQUEUE_NET_MAX_SIZE) {
            log_error("process_descriptor_chain failed");
            goto free_iov;
        }
        vnet_header = iov[0].iov_base;
        iov_packet = rm_iov_header(iov, &n, header_len);
        if (iov_packet == NULL)
            goto free_iov;
        // Read a packet from tap device
        len = readv(net->tapfd, iov_packet, n);

        if (len < 0 && errno == EWOULDBLOCK) {
            // No more packets from tapfd, restore last_avail_idx.
            log_info("no more packets");
            vq->last_avail_idx--;
            free(iov);
            break;
        }

        memset(vnet_header, 0, header_len);
        if (vdev->regs.drv_feature & (1ULL << VIRTIO_F_VERSION_1)) {
            ((NetHdr *)vnet_header)->num_buffers = 1;
        }

        update_used_ring(vq, idx, len + header_len);
        free(iov);
    }

    virtio_inject_irq(vq);
    return;
free_iov:
    free(iov);
}

static void virtq_tx_handle_one_request(VirtIODevice *vdev, VirtQueue *vq) {
    struct iovec *iov = NULL;
    int i, n;
    int packet_len, all_len; // all_len include the header length.
    uint16_t idx;
    static char pad[64];
    ssize_t len;
    NetDev *net = vdev->dev;
    size_t header_len = get_nethdr_size(vdev);
    if (net->tapfd == -1) {
        log_error("tap device is invalid");
        return;
    }

    n = process_descriptor_chain(vq, &idx, &iov, NULL, 1, false);
    if (n < 1) {
        return;
    }

    for (i = 0, all_len = 0; i < n; i++)
        all_len += iov[i].iov_len;

    packet_len = all_len - header_len;
    iov[0].iov_base += header_len;
    iov[0].iov_len -= header_len;
    log_debug("packet send: %d bytes", packet_len);

    // The mininum packet for data link layer is 64 bytes.
    if (packet_len < 64) {
        iov[n].iov_base = pad;
        iov[n].iov_len = 64 - packet_len;
        n++;
    }
    len = writev(net->tapfd, iov, n);
    if (len < 0) {
        log_error("write tap failed, errno %d", errno);
    }
    update_used_ring(vq, idx, all_len);
    free(iov);
}

int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("virtio_net_txq_notify_handler");
    virtqueue_disable_notify(vq);
    while (!virtqueue_is_empty(vq)) {
        virtq_tx_handle_one_request(vdev, vq);
    }
    virtqueue_enable_notify(vq);
    // TODO: Can we don't inject irq when send packets to improve performance?
    // Linux will recycle the used ring when send packets.
    // virtio_inject_irq(vq);
    return 0;
}

int virtio_net_init(VirtIODevice *vdev, char *devname) {
    log_info("virtio net init");
    NetDev *net = vdev->dev;
    // open tap device
    net->tapfd = open_tap(devname);
    if (net->tapfd == -1) {
        log_error("open tap device failed");
        return -1;
    }
    // set tap device O_NONBLOCK. If io operation like readv blocks, then return
    // errno EWOULDBLOCK
    if (set_nonblocking(net->tapfd) < 0) {
        close(net->tapfd);
        net->tapfd = -1;
    }
    // register an epoll read event for tap device
    net->event = add_event(net->tapfd, EPOLLIN, virtio_net_event_handler, vdev);
    if (net->event == NULL) {
        log_error("Can't register net event");
        close(net->tapfd);
        net->tapfd = -1;
        return -1;
    }
    vdev->virtio_close = virtio_net_close;
    return 0;
}

void virtio_net_close(VirtIODevice *vdev) {
    NetDev *dev = vdev->dev;
    close(dev->tapfd);
    free(dev->event);
    free(dev);
    free(vdev->vqs);
    free(vdev);
}
