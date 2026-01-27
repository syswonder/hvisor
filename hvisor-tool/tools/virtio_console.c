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
#define _GNU_SOURCE

#include "virtio_console.h"
#include "log.h"
#include "virtio.h"
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include <termios.h>

static uint8_t trashbuf[1024];

ConsoleDev *init_console_dev() {
    ConsoleDev *dev = (ConsoleDev *)malloc(sizeof(ConsoleDev));
    dev->config.cols = 80;
    dev->config.rows = 25;
    dev->master_fd = -1;
    dev->rx_ready = -1;
    dev->event = NULL;
    return dev;
}

static void virtio_console_event_handler(int fd, int epoll_type, void *param) {
    // log_debug("%s", __func__);
    VirtIODevice *vdev = (VirtIODevice *)param;
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;
    VirtQueue *vq = &vdev->vqs[CONSOLE_QUEUE_RX];
    int n;
    ssize_t len;
    struct iovec *iov = NULL;
    uint16_t idx;

    if (epoll_type != EPOLLIN || fd != dev->master_fd) {
        log_error("Invalid console event");
        return;
    }
    if (dev->master_fd <= 0 || vdev->type != VirtioTConsole) {
        log_error("console event handler should not be called");
        return;
    }
    if (dev->rx_ready <= 0) {
        read(dev->master_fd, trashbuf, sizeof(trashbuf));
        return;
    }
    if (virtqueue_is_empty(vq)) {
        read(dev->master_fd, trashbuf, sizeof(trashbuf));
        virtio_inject_irq(vq);
        return;
    }

    while (!virtqueue_is_empty(vq)) {
        n = process_descriptor_chain(vq, &idx, &iov, NULL, 0, false);
        if (n < 1) {
            log_error("process_descriptor_chain failed");
            break;
        }
        len = readv(dev->master_fd, iov, n);
        if (len < 0 && errno == EWOULDBLOCK) {
            log_debug("no more bytes");
            vq->last_avail_idx--;
            free(iov);
            break;
        } else if (len < 0) {
            log_trace("Failed to read from console, errno is %d", errno);
            vq->last_avail_idx--;
            free(iov);
            break;
        }
        update_used_ring(vq, idx, len);
        free(iov);
    }
    virtio_inject_irq(vq);
    return;
}

int virtio_console_init(VirtIODevice *vdev) {
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;
    int master_fd, slave_fd;
    char *slave_name;
    struct termios term_io;

    master_fd = posix_openpt(O_RDWR | O_NOCTTY);
    if (master_fd < 0) {
        log_error("Failed to open master pty, errno is %d", errno);
    }
    if (grantpt(master_fd) < 0) {
        log_error("Failed to grant pty, errno is %d", errno);
    }
    if (unlockpt(master_fd) < 0) {
        log_error("Failed to unlock pty, errno is %d", errno);
    }
    dev->master_fd = master_fd;

    slave_name = ptsname(master_fd);
    if (slave_name == NULL) {
        log_error("Failed to get slave name, errno is %d", errno);
    }
    log_info("char device redirected to %s", slave_name);
    // Disable line discipline to prevent the TTY
    // from echoing the characters sent from the master back to the master.
    slave_fd = open(slave_name, O_RDWR);
    tcgetattr(slave_fd, &term_io);
    cfmakeraw(&term_io);
    tcsetattr(slave_fd, TCSAFLUSH, &term_io);
    close(slave_fd);

    if (set_nonblocking(dev->master_fd) < 0) {
        dev->master_fd = -1;
        close(dev->master_fd);
        log_error("Failed to set nonblocking mode, fd closed!");
    }

    dev->event =
        add_event(dev->master_fd, EPOLLIN, virtio_console_event_handler, vdev);

    if (dev->event == NULL) {
        log_error("Can't register console event");
        close(master_fd);
        dev->master_fd = -1;
        return -1;
    }

    vdev->virtio_close = virtio_console_close;
    return 0;
}

int virtio_console_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("%s", __func__);
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;
    if (dev->rx_ready <= 0) {
        dev->rx_ready = 1;
        virtqueue_disable_notify(vq);
    }
    return 0;
}

static void virtq_tx_handle_one_request(ConsoleDev *dev, VirtQueue *vq) {
    int n;
    uint16_t idx;
    ssize_t len;
    struct iovec *iov = NULL;
    if (dev->master_fd <= 0) {
        log_error("Console master fd is not ready");
        return;
    }

    n = process_descriptor_chain(vq, &idx, &iov, NULL, 0, false);

    if (n < 1) {
        return;
    }

    len = writev(dev->master_fd, iov, n);
    if (len < 0) {
        log_error("Failed to write to console, errno is %d", errno);
    }
    update_used_ring(vq, idx, 0);
    free(iov);
}

int virtio_console_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("%s", __func__);
    while (!virtqueue_is_empty(vq)) {
        virtqueue_disable_notify(vq);
        while (!virtqueue_is_empty(vq)) {
            virtq_tx_handle_one_request(vdev->dev, vq);
        }
        virtqueue_enable_notify(vq);
    }
    virtio_inject_irq(vq);
    return 0;
}

void virtio_console_close(VirtIODevice *vdev) {
    ConsoleDev *dev = vdev->dev;
    close(dev->master_fd);
    free(dev->event);
    free(dev);
    free(vdev->vqs);
    free(vdev);
}