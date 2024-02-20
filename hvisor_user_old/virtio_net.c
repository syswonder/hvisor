#include "virtio_net.h"
#include "log.h"
#include "mevent.h"
#include "virtio.h"
#include <stdlib.h>
#include <net/if.h>
#include <fcntl.h>
#include <string.h>
#include <linux/if_tun.h>
#include <sys/ioctl.h>
#include <unistd.h>
#include <sys/uio.h>

static uint8_t dummybuf[2048];

NetDev *init_net_dev(uint8_t mac[])
{
    NetDev *dev = malloc(sizeof(NetDev));
    dev->config.mac[0] = mac[0];
    dev->config.mac[1] = mac[1];
    dev->config.mac[2] = mac[2];
    dev->config.mac[3] = mac[3];
    dev->config.mac[4] = mac[4];
    dev->config.mac[5] = mac[5];
    dev->config.status = VIRTIO_NET_S_LINK_UP;
    dev->tapfd = -1;
    dev->mevp = NULL;
}
// open tap device
static int virtio_net_tap_open(char *devname)
{
    char tbuf[IFNAMSIZ] = "/dev/net/tun";
    int tunfd, rc;
    struct ifreq ifr;
    tunfd = open(tbuf, O_RDWR);
    if (tunfd < 0) {
        log_error("Failed to open tap device");
        return -1;
    }
    memset(&ifr, 0, sizeof(ifr));
    // IFF_NO_PI tells kernel do not provide message header
    ifr.ifr_flags = IFF_TAP | IFF_NO_PI;
    strncpy(ifr.ifr_name, devname, IFNAMSIZ);
    ifr.ifr_name[IFNAMSIZ - 1] = '\0';
    rc = ioctl(tunfd, TUNSETIFF, (void *)&ifr);
    if (rc < 0) {
        log_error("open of tap device %s fail", devname);
        close(tunfd);
        return -1;
    }
    strncpy(devname, ifr.ifr_name, IFNAMSIZ);
    return tunfd;
}
/// Called when tap device received packets
static void virtio_net_rx_callback(int fd, enum ev_type type, void *param)
{
    VirtIODevice *vdev = param;

}

int virtio_net_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{

}

static void virtio_net_tap_rx(VirtIODevice *vdev)
{
    struct iovec iov[VIRTQUEUE_NET_MAX_SIZE];
    NetDev *net = vdev->dev;
    VirtQueue *rx_vq = vdev->vqs[VIRTIO_NET_RXQ];

    if (net->tapfd == -1 || vdev->type != VirtioTNet) {
        log_error("tap rx is wrong");
        return;
    }
    // if rx_vq is not setup, drop the packet
    if (!rx_vq->ready) {
        read(net->tapfd, dummybuf, sizeof(dummybuf));
        return;
    }

    if (virtqueue_is_empty(rx_vq)) {
        read(net->tapfd, dummybuf, sizeof(dummybuf));
        return;
    }



}

/// Send iov to tap device.
static void virtio_net_tap_tx(NetDev *net, struct iovec *iov, int iovcnt, int len)
{
    static char pad[60]; /* all zero bytes */

    if (net->tapfd == -1) {
        log_error("tap device is invalid");
        return;
    }

    /*
     * If the length is < 60, pad out to that and add the
     * extra zero'd segment to the iov. It is guaranteed that
     * there is always an extra iov available by the caller.
     */
    if (len < 60) {
        iov[iovcnt].iov_base = pad;
        iov[iovcnt].iov_len = 60 - len;
        iovcnt++;
    }
    writev(net->tapfd, iov, iovcnt);
}

static void virtq_tx_handle_one_request(NetDev *net, VirtQueue *vq)
{
    struct iovec iov[VIRTQUEUE_NET_MAX_SIZE + 1];
    int i, n;
    int plen, tlen; // packet length and transfer length.
    uint16_t idx;

    n = vq_getchain(vq, &idx, iov, VIRTQUEUE_NET_MAX_SIZE, NULL);
    if (n < 1)
        return ;
    plen = 0;
    tlen = iov[0].iov_len;
    for (i = 1; i < n; i++) {
        plen += iov[i].iov_len;
        tlen += iov[i].iov_len;
    }
    log_info("virtio: packet send, %d bytes, %d segs\n\r", plen, n);
    virtio_net_tap_tx(net, &iov[1], n-1, plen);
    update_used_ring(vq, idx, tlen);
}

int virtio_net_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{
    virtqueue_disable_notify(vq);
    while(!virtqueue_is_empty(vq)) {
        virtq_tx_handle_one_request(vdev->dev, vq);
    }
    virtqueue_enable_notify(vq);
    return 0;
}


int virtio_net_init(VirtIODevice *vdev, char *devname)
{
    NetDev *net = vdev->dev;
    // open tap device
    net->tapfd = virtio_net_tap_open(devname);
    if( net->tapfd == -1 ) {
        log_error("open tap device failed");
        return -1;
    }
    // set tap device O_NONBLOCK
    int opt = 1;
    if (ioctl(net->tapfd, FIONBIO, &opt) < 0) {
        log_error("tap device O_NONBLOCK failed");
        close(net->tapfd);
        net->tapfd = -1;
    }
    // register an epoll read event for tap device
    net->mevp = mevent_add(net->tapfd, EVF_READ, virtio_net_rx_callback, vdev);
    if (net->mevp == NULL) {
        log_error("Can't register net mevp");
        close(net->tapfd);
        net->tapfd = -1;
        return -1;
    }
    return 0;
}
