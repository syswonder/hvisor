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

#include "virtio_blk.h"
#include "log.h"
#include "virtio.h"
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/param.h>
#include <sys/stat.h>

/*
 * Threading model
 * ---------------
 * Two threads touch a virtio-blk device:
 *
 *   main thread   (epoll loop in virtio.c)
 *     - calls notify_handler when the guest kicks the virtqueue.
 *     - calls virtio_blk_close on shutdown.
 *     - does NOT touch the virtqueue or BlkDev (except mtx/cond/close).
 *
 *   worker thread (blkproc_thread, one per blk device)
 *     - owns the virtqueue exclusively: drains avail_ring, performs disk I/O,
 *       updates used_ring, and injects IRQs back to the guest.
 *     - only it reads/writes vq->last_avail_idx and vq->last_used_idx.
 *
 * The virtqueue (avail_ring, desc_table) is single-threaded - the main thread
 * never accesses it. This avoids the intermediate procq and the extra locking
 * the old design required.
 *
 * Cross-CPU shared memory (guest <-> worker)
 * ------------------------------------------
 * avail_ring->idx is written by the guest and read by the worker, hence the
 * ACQUIRE load in vq_is_empty(). used_ring is written by the worker and read
 * by the guest, hence write_barrier() in update_used_ring().
 */

/**
 * VIRTIO_BLK_T_IN — read sectors from the backing file.
 *
 * Virtio descriptor layout: out_iov=[header], in_iov=[data…, status].
 *
 * @param fd    backing file descriptor
 * @param wlen  [out] bytes successfully read
 * @param iov   guest data buffers (device-writable in_iov)
 * @param cnt   number of iov entries
 * @param off   byte offset (= sector * 512)
 * @return      0 on success, errno on failure
 */
static int blk_do_read(int fd, ssize_t *wlen, struct iovec *iov, int cnt,
                       uint64_t off) {
    ssize_t len = preadv(fd, iov, cnt, off);
    log_debug("preadv, len=%zd, offset=%ld", len, off);
    if (len < 0) {
        log_error("preadv failed, errno=%d", errno);
        return errno;
    }
    *wlen = len;
    return 0;
}

/**
 * VIRTIO_BLK_T_OUT — write sectors to the backing file.
 *
 * Virtio descriptor layout: out_iov=[header, data…], in_iov=[status].
 *
 * @param fd   backing file descriptor
 * @param iov  guest data buffers (device-readable out_iov, excluding header)
 * @param cnt  number of iov entries
 * @param off  byte offset (= sector * 512)
 * @return     0 on success, errno on failure
 */
static int blk_do_write(int fd, struct iovec *iov, int cnt, uint64_t off) {
    ssize_t len = pwritev(fd, iov, cnt, off);
    log_debug("pwritev, len=%zd, offset=%ld", len, off);
    if (len < 0) {
        log_error("pwritev failed, errno=%d", errno);
        return errno;
    }
    return 0;
}

/**
 * VIRTIO_BLK_T_FLUSH — persist all previously completed writes.
 *
 * Virtio descriptor layout: out_iov=[header], in_iov=[status].
 * Implemented via fdatasync(2); guarantees data is on stable storage.
 *
 * @param fd  backing file descriptor
 * @return    0 on success, errno on failure
 */
static int blk_do_flush(int fd) {
    if (fdatasync(fd) < 0) {
        log_error("fdatasync failed, errno=%d", errno);
        return errno;
    }
    return 0;
}

/**
 * VIRTIO_BLK_T_GET_ID — return the device identification string.
 *
 * Virtio descriptor layout: out_iov=[header], in_iov=[id_buf, status].
 * The string is NUL-terminated unless the buffer is exactly 20 bytes
 * (VIRTIO_BLK_ID_BYTES).
 *
 * @param iov  guest ID buffer (first in_iov entry)
 * @return     number of bytes written (= strlen + 1, capped at iov_len)
 */
static ssize_t blk_do_get_id(struct iovec *iov) {
    int n = snprintf(iov->iov_base, iov->iov_len, "hvisor-virblk");
    return MIN(n + 1, (ssize_t)iov->iov_len);
}

/**
 * Set the status byte and push a used-ring entry.
 *
 * Every consumed descriptor — including corrupt ones, which are completed
 * with @p wlen = 0 — must call this to keep avail- and used-ring indices
 * in sync.  The used-ring length is @p wlen + 1 to account for the status
 * byte itself.
 *
 * @param vq    target virtqueue
 * @param idx   descriptor index (id field in used-ring element)
 * @param st    pointer to the status byte in guest memory (may be NULL if
 *              the descriptor chain was malformed and no status byte exists)
 * @param err   0 for success, EOPNOTSUPP, or an errno value
 * @param wlen  data bytes transferred (0 for FLUSH, discard, or errors)
 */
static void blk_complete(VirtQueue *vq, uint16_t idx, uint8_t *st, int err,
                         ssize_t wlen) {
    if (st) {
        if (err == 0)
            *st = VIRTIO_BLK_S_OK;
        else if (err == EOPNOTSUPP)
            *st = VIRTIO_BLK_S_UNSUPP;
        else
            *st = VIRTIO_BLK_S_IOERR;
    }
    if (err && err != EOPNOTSUPP)
        log_error("virtio-block error, err=%d", err);
    update_used_ring(vq, idx, wlen + 1);
}

static void virtq_blk_handle_one_request(BlkDev *dev, VirtQueue *vq) {
    struct VirtioBufConfig cfg = {
        .out_iov = dev->out_buf,
        .max_out = VIRTQUEUE_BLK_MAX_SIZE,
        .in_iov = dev->in_buf,
        .max_in = VIRTQUEUE_BLK_MAX_SIZE,
    };
    uint16_t desc_idx =
        vq->avail_ring->ring[vq->last_avail_idx & (vq->num - 1)];

    struct VirtioRequest vreq;
    int ret = process_descriptor_chain_buf(vq, desc_idx, &cfg, &vreq);
    if (ret <= 0) {
        log_error("failed to process descriptor chain, ret=%d", ret);
        vq->last_avail_idx++;
        blk_complete(vq, desc_idx, NULL, EIO, 0);
        return;
    }

    if (vreq.out_count < 1 || vreq.out_iov[0].iov_len != sizeof(BlkReqHead)) {
        log_error("invalid header");
        blk_complete(vq, desc_idx, NULL, EIO, 0);
        return;
    }

    if (vreq.in_count < 1 || vreq.in_iov[vreq.in_count - 1].iov_len != 1) {
        log_error("invalid status byte");
        blk_complete(vq, desc_idx, NULL, EIO, 0);
        return;
    }

    BlkReqHead *hdr = vreq.out_iov[0].iov_base;
    uint8_t *vstatus = vreq.in_iov[vreq.in_count - 1].iov_base;
    int err = 0;
    ssize_t wlen = 0;

    switch (hdr->type) {
    case VIRTIO_BLK_T_IN:
        err = blk_do_read(dev->img_fd, &wlen, vreq.in_iov, vreq.in_count - 1,
                          hdr->sector * SECTOR_BSIZE);
        break;
    case VIRTIO_BLK_T_OUT:
        err = blk_do_write(dev->img_fd, &vreq.out_iov[1], vreq.out_count - 1,
                           hdr->sector * SECTOR_BSIZE);
        break;
    case VIRTIO_BLK_T_FLUSH:
        err = blk_do_flush(dev->img_fd);
        break;
    case VIRTIO_BLK_T_GET_ID:
        wlen = blk_do_get_id(&vreq.in_iov[0]);
        break;
    default:
        err = EOPNOTSUPP;
        log_error("unsupported operation type %u", hdr->type);
        break;
    }

    blk_complete(vq, desc_idx, vstatus, err, wlen);
}

/*
 * Worker thread entry point - one per virtio-blk device.
 *
 * The worker is the sole owner of the virtqueue:
 *   1. Wait on cond until notify_handler signals or close is set.
 *   2. Drain the avail_ring in a disable-notify / process / enable-notify
 *      loop to suppress redundant guest notifications while we're busy.
 *   3. Inject a single IRQ after each batch to tell the guest about
 *      completed requests.
 *   4. Loop back to step 1.
 *
 * virtio_inject_irq() is only called when the queue was non-empty, so
 * used_ring is guaranteed to be valid (set up by the guest before the
 * first kick).
 */
static void *blkproc_thread(void *arg) {
    VirtIODevice *vdev = (VirtIODevice *)arg;
    BlkDev *dev = vdev->dev;
    VirtQueue *vq = vdev->vqs;

    for (bool closing = false; !closing;) {
        // Hold mtx to check the close flag and wait on cond.
        pthread_mutex_lock(&dev->mtx);
        while (vq_is_empty(vq) && !dev->close)
            pthread_cond_wait(&dev->cond, &dev->mtx);
        closing = dev->close;
        pthread_mutex_unlock(&dev->mtx);

        // Drain all pending requests. The double-checked loop follows the
        // standard virtio pattern: disable-notify, process until empty,
        // enable-notify, then re-check in case the guest added buffers
        // while notifications were suppressed.
        if (!vq_is_empty(vq)) {
            do {
                virtqueue_disable_notify(vq);
                while (!vq_is_empty(vq))
                    virtq_blk_handle_one_request(dev, vq);
                virtqueue_enable_notify(vq);
            } while (!vq_is_empty(vq));

            // Tell the guest that used-ring entries are available.
            virtio_inject_irq(vq);
        }
    }

    pthread_exit(NULL);
    return NULL;
}

// create blk dev.
static BlkDev *init_blk_dev(VirtIODevice *vdev) {
    BlkDev *dev = malloc(sizeof(BlkDev));
    vdev->dev = dev;
    dev->config.capacity = -1;
    dev->config.size_max = -1;
    dev->config.seg_max = BLK_SEG_MAX;
    dev->config.blk_size = SECTOR_BSIZE;
    dev->img_fd = -1;
    dev->close = false;
    pthread_mutex_init(&dev->mtx, NULL);
    pthread_cond_init(&dev->cond, NULL);
    pthread_create(&dev->tid, NULL, blkproc_thread, vdev);
    return dev;
}

static int virtio_blk_init(VirtIODevice *vdev, const char *img_path) {
    BlkDev *dev = vdev->dev;
    if (!dev) {
        log_error("virtio_blk_init: vdev->dev is nullptr");
        return -1;
    }

    dev->img_fd = open(img_path, O_RDWR);
    if (dev->img_fd == -1) {
        log_error("cannot open %s, Error code is %d", img_path, errno);
        return -1;
    }

    struct stat st;
    if (fstat(dev->img_fd, &st) == -1) {
        log_error("cannot stat %s, Error code is %d", img_path, errno);
        return -1;
    }
    uint64_t blk_size = st.st_size / SECTOR_BSIZE;
    dev->config.capacity = blk_size;
    dev->config.size_max = blk_size;

    log_info("virtio_blk_init: %s, size is %" PRIu64, img_path,
             dev->config.capacity);
    return 0;
}

/*
 * Called by the main thread when the guest writes to the queue_notify MMIO
 * register. Wakes up the worker thread so it can drain the virtqueue.
 */
static int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    BlkDev *dev = vdev->dev;
    (void)vq;

    // Wake up the worker thread. mtx pairs with the worker's cond_wait.
    pthread_mutex_lock(&dev->mtx);
    pthread_cond_signal(&dev->cond);
    pthread_mutex_unlock(&dev->mtx);
    return 0;
}

static void virtio_blk_reset(VirtIODevice *vdev) { (void)vdev; }

/*
 * Shut down the blk device: signal close, wait for the worker to exit,
 * then release all resources.
 */
static void virtio_blk_close(VirtIODevice *vdev) {
    if (!vdev)
        return;

    BlkDev *dev = vdev->dev;
    if (dev) {
        pthread_mutex_lock(&dev->mtx);
        dev->close = true;
        pthread_cond_signal(&dev->cond);
        pthread_mutex_unlock(&dev->mtx);
        pthread_join(dev->tid, NULL);
        pthread_mutex_destroy(&dev->mtx);
        pthread_cond_destroy(&dev->cond);
        if (dev->img_fd >= 0)
            close(dev->img_fd);
        free(dev);
        vdev->dev = NULL;
    }
    free(vdev->vqs);
    vdev->vqs = NULL;
    free(vdev);
}

static int virtio_blk_do_init(VirtIODevice *vdev, const void *params) {
    const struct virtio_blk_init_params *p = params;
    if (!p)
        return -EINVAL;
    if (!init_blk_dev(vdev))
        return -ENOMEM;
    if (virtio_blk_init(vdev, p->img_path) != 0)
        return -EIO;
    return 0;
}

const struct virtio_device_ops virtio_blk_ops = {
    .type = VirtioTBlock,
    .features = BLK_SUPPORTED_FEATURES,
    .num_queues = 1,
    .queue_max_size = VIRTQUEUE_BLK_MAX_SIZE,
    .init = virtio_blk_do_init,
    .close = virtio_blk_close,
    .reset = virtio_blk_reset,
    .notify_handlers = {virtio_blk_notify_handler},
};

static int virtio_blk_parse_params(const cJSON *json, void **out) {
    struct virtio_blk_init_params *p = calloc(1, sizeof(*p));
    if (!p)
        return -ENOMEM;
    cJSON *img = cJSON_GetObjectItem(json, "img");
    if (!cJSON_IsString(img) || !img->valuestring[0]) {
        free(p);
        return -EINVAL;
    }
    p->img_path = img->valuestring;
    *out = p;
    return 0;
}

static void virtio_blk_free_params(void *params) { free(params); }

const struct virtio_config_ops virtio_blk_config_ops = {
    .parse = virtio_blk_parse_params,
    .free = virtio_blk_free_params,
};
