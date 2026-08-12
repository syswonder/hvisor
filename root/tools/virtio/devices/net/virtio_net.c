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
#include "json_parse.h"
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
    dev->in_iov = NULL;
    dev->out_iov = NULL;
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
    // IFF_VNET_HDR enables virtio-net header passthrough with TAP
    ifr.ifr_flags = IFF_TAP | IFF_NO_PI | IFF_VNET_HDR;
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
    NetDev *net = vdev->dev;
    VirtQueue *vq = &vdev->vqs[NET_QUEUE_RX];
    ssize_t len;
    if (fd != net->tapfd || !(epoll_type & EPOLLIN)) {
        log_error("invalid event");
        return;
    }
    if (net->tapfd == -1 || vdev->type != VirtioTNet) {
        log_error("net rx callback should not be called");
        return;
    }

    // if vq is not setup, drop the packet
    uint8_t trashbuf[1600];
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
    uint16_t batch_indices[VIRTQUEUE_NET_MAX_SIZE];
    uint32_t batch_lens[VIRTQUEUE_NET_MAX_SIZE];
    size_t batch_count = 0;
    while (!virtqueue_is_empty(vq)) {
        struct VirtioBufConfig cfg = {
            .in_iov = net->in_iov,
            .max_in = NET_IOV_MAX,
        };
        struct VirtioRequest req;
        uint16_t idx = vq->avail_ring->ring[vq->last_avail_idx & (vq->num - 1)];
        int n = process_descriptor_chain_buf(vq, idx, &cfg, &req);
        if (n < 1 || n > VIRTQUEUE_NET_MAX_SIZE) {
            log_error("process_descriptor_chain_buf failed: %d", n);
            if (n < 1) {
                vq->last_avail_idx++;
            }
            batch_indices[batch_count] = idx;
            batch_lens[batch_count] = 0;
            batch_count++;
            break;
        }

        // RX: all buffers are VRING_DESC_F_WRITE → in_iov
        // IFF_VNET_HDR: TAP writes [virtio_net_hdr | packet] directly
        len = readv(net->tapfd, req.in_iov, req.in_count);

        if (len < 0 && errno == EWOULDBLOCK) {
            // No more packets from tapfd, restore last_avail_idx.
            log_info("no more packets");
            vq->last_avail_idx--;
            break;
        }

        if (len < 0) {
            log_error("readv from tap failed, errno %d", errno);
            batch_indices[batch_count] = idx;
            batch_lens[batch_count] = 0;
            batch_count++;
            break;
        }

        if (len == 0) {
            log_error("tap device EOF (closed or bridge down)");
            batch_indices[batch_count] = idx;
            batch_lens[batch_count] = 0;
            batch_count++;
            net->rx_ready = 0;
            break;
        }

        batch_indices[batch_count] = idx;
        batch_lens[batch_count] = len;
        batch_count++;
    }

    if (batch_count > 0)
        update_used_ring_batch(vq, batch_indices, batch_lens, batch_count);
    virtio_inject_irq(vq);
}

static void virtq_tx_handle_one_request(VirtIODevice *vdev, VirtQueue *vq,
                                        uint16_t *out_indices,
                                        uint32_t *out_lens, size_t *out_count) {
    NetDev *net = vdev->dev;
    if (net->tapfd == -1) {
        log_error("tap device is invalid");
        return;
    }

    size_t header_len = get_nethdr_size(vdev);
    struct VirtioBufConfig cfg = {
        .out_iov = net->out_iov,
        .max_out = NET_IOV_MAX - 1,
    };
    struct VirtioRequest req;
    uint16_t idx = vq->avail_ring->ring[vq->last_avail_idx & (vq->num - 1)];

    int n = process_descriptor_chain_buf(vq, idx, &cfg, &req);
    if (n < 1) {
        log_error("process_descriptor_chain_buf failed: %d", n);
        vq->last_avail_idx++;
        out_indices[*out_count] = idx;
        out_lens[*out_count] = 0;
        (*out_count)++;
        return;
    }

    // TX: no descriptors are VRING_DESC_F_WRITE → out_iov
    if ((size_t)req.out_iov[0].iov_len < header_len) {
        log_error("malformed TX packet: iov[0] too small for header");
        out_indices[*out_count] = idx;
        out_lens[*out_count] = 0;
        (*out_count)++;
        return;
    }

    size_t all_len = 0;
    for (size_t i = 0; i < req.out_count; i++)
        all_len += req.out_iov[i].iov_len;

    size_t packet_len = all_len - header_len;
    log_debug("packet send: %zu bytes", packet_len);

    // The mininum packet for data link layer is 64 bytes.
    char pad[64] = {0};
    if (packet_len < 64) {
        req.out_iov[req.out_count].iov_base = pad;
        req.out_iov[req.out_count].iov_len = 64 - packet_len;
        req.out_count++;
    }
    ssize_t len = writev(net->tapfd, req.out_iov, req.out_count);
    if (len < 0) {
        log_error("write tap failed, errno %d", errno);
    }
    out_indices[*out_count] = idx;
    out_lens[*out_count] = (len < 0) ? 0 : all_len;
    (*out_count)++;
}

int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("virtio_net_txq_notify_handler");
    virtqueue_disable_notify(vq);
    uint16_t batch_indices[VIRTQUEUE_NET_MAX_SIZE];
    uint32_t batch_lens[VIRTQUEUE_NET_MAX_SIZE];
    size_t batch_count = 0;
    for (;;) {
        while (!virtqueue_is_empty(vq)) {
            virtq_tx_handle_one_request(vdev, vq, batch_indices, batch_lens,
                                        &batch_count);
        }
        if (batch_count > 0) {
            update_used_ring_batch(vq, batch_indices, batch_lens, batch_count);
            batch_count = 0;
            virtio_inject_irq(vq);
        }
        virtqueue_enable_notify(vq);
        // Re-check: guest may have added descriptors between our last
        // empty check and enable_notify.  Without this, UDP streams
        // lose descriptors permanently because each sendto() issues
        // exactly one kick and the suppressed notification never fires.
        if (!virtqueue_is_empty(vq)) {
            virtqueue_disable_notify(vq);
            continue;
        }
        break;
    }
    return 0;
}

void net_on_status(VirtIODevice *vdev, uint32_t status) {
    NetDev *net = vdev->dev;

    // FEATURES_OK indicates guest has finished writing DRIVER_FEATURES.
    // Configure TAP virtio-net header size to match the negotiated format.
    if (status & VIRTIO_CONFIG_S_FEATURES_OK) {
        int hdr_sz = (int)get_nethdr_size(vdev);
        if (ioctl(net->tapfd, TUNSETVNETHDRSZ, &hdr_sz) < 0)
            log_error("TUNSETVNETHDRSZ(%d) failed", hdr_sz);
    }

    if (status == 0) {
        net->rx_ready = 0;
    }
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
    net->in_iov = malloc(sizeof(struct iovec) * NET_IOV_MAX);
    net->out_iov = malloc(sizeof(struct iovec) * NET_IOV_MAX);
    if (!net->in_iov || !net->out_iov) {
        log_error("failed to allocate iov buffers");
        free(net->in_iov);
        free(net->out_iov);
        net->in_iov = NULL;
        net->out_iov = NULL;
        close(net->tapfd);
        net->tapfd = -1;
        return -1;
    }
    return 0;
}

void virtio_net_reset(VirtIODevice *vdev) {
    if (!vdev || !vdev->dev)
        return;
    NetDev *dev = vdev->dev;
    dev->rx_ready = false;
}

void virtio_net_close(VirtIODevice *vdev) {
    NetDev *dev = vdev->dev;
    close(dev->tapfd);
    free(dev->event);
    free(dev->in_iov);
    free(dev->out_iov);
    free(dev);
    free(vdev->vqs);
    free(vdev);
}

static int virtio_net_do_init(VirtIODevice *vdev, void *params) {
    const struct virtio_net_init_params *p = params;
    if (!p)
        return -EINVAL;
    vdev->dev = init_net_dev((uint8_t *)p->mac);
    if (!vdev->dev)
        return -ENOMEM;
    return virtio_net_init(vdev, (char *)p->tap);
}

const struct virtio_device_ops virtio_net_ops = {
    .type = VirtioTNet,
    .features = NET_SUPPORTED_FEATURES,
    .num_queues = NET_MAX_QUEUES,
    .queue_max_size = VIRTQUEUE_NET_MAX_SIZE,
    .init = virtio_net_do_init,
    .close = virtio_net_close,
    .reset = virtio_net_reset,
    .status_changed = net_on_status,
    .notify_handlers =
        {
            [NET_QUEUE_RX] = virtio_net_rxq_notify_handler,
            [NET_QUEUE_TX] = virtio_net_txq_notify_handler,
        },
};

static int virtio_net_parse_params(cJSON *json, void **out) {
    struct virtio_net_init_params *p = calloc(1, sizeof(*p));
    if (!p)
        return -ENOMEM;

    cJSON *tap = cJSON_GetObjectItem(json, "tap");
    if (!cJSON_IsString(tap) || !tap->valuestring[0]) {
        free(p);
        return -EINVAL;
    }
    p->tap = tap->valuestring;

    cJSON *mac_json = cJSON_GetObjectItem(json, "mac");
    if (cJSON_GetArraySize(mac_json) != 6) {
        free(p);
        return -EINVAL;
    }
    for (int i = 0; i < 6; i++) {
        if (parse_json_u8(cJSON_GetArrayItem(mac_json, i), &p->mac[i]) != 0) {
            log_error("failed to parse mac byte %d", i);
            free(p);
            return -EINVAL;
        }
    }

    *out = p;
    return 0;
}

static void virtio_net_free_params(void *params) { free(params); }

const struct virtio_config_ops virtio_net_config_ops = {
    .parse = virtio_net_parse_params,
    .free = virtio_net_free_params,
};
