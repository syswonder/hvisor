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
#include <stdlib.h>
#include <string.h>
#include <sys/param.h>
#include <sys/stat.h>

static void complete_block_operation(BlkDev *dev, struct blkp_req *req,
                                     VirtQueue *vq, int err,
                                     ssize_t written_len) {
    uint8_t *vstatus = (uint8_t *)(req->iov[req->iovcnt - 1].iov_base);
    int is_empty = 0;
    if (err == EOPNOTSUPP)
        *vstatus = VIRTIO_BLK_S_UNSUPP;
    else if (err != 0)
        *vstatus = VIRTIO_BLK_S_IOERR;
    else
        *vstatus = VIRTIO_BLK_S_OK;
    if (err != 0) {
        log_error("virt blk err, num is %d", err);
    }
    update_used_ring(vq, req->idx, written_len + 1);
    pthread_mutex_lock(&dev->mtx);
    is_empty = TAILQ_EMPTY(&dev->procq);
    pthread_mutex_unlock(&dev->mtx);
    if (is_empty)
        virtio_inject_irq(vq);
    free(req->iov);
    free(req);
}
// get a blk req from procq
static int get_breq(BlkDev *dev, struct blkp_req **req) {
    struct blkp_req *elem;
    elem = TAILQ_FIRST(&dev->procq);
    if (elem == NULL) {
        return 0;
    }
    TAILQ_REMOVE(&dev->procq, elem, link);
    *req = elem;
    return 1;
}

static void blkproc(BlkDev *dev, struct blkp_req *req, VirtQueue *vq) {
    struct iovec *iov = req->iov;
    int n = req->iovcnt, err = 0;
    ssize_t len, written_len = 0;

    switch (req->type) {
    case VIRTIO_BLK_T_IN:
        written_len = len = preadv(dev->img_fd, &iov[1], n - 2, req->offset);
        // log_debug("readv data is ");
        // for(int i = 1; i < n-1; i++) {
        //     log_debug("n-1 is %d, iov[i].iov_len is %d", n-1,
        //     iov[i].iov_len); for (int j = 0; j < iov[i].iov_len; j++)
        //         printf("%x", *(int*)(iov[i].iov_base + j));
        //     printf("\n");
        // }
        log_debug("preadv, len is %d, offset is %d", len, req->offset);
        if (len < 0) {
            log_error("pread failed");
            err = errno;
        }
        break;
    case VIRTIO_BLK_T_OUT:
        len = pwritev(dev->img_fd, &iov[1], n - 2, req->offset);
        log_debug("pwritev, len is %d, offset is %d", len, req->offset);
        if (len < 0) {
            log_error("pwrite failed");
            err = errno;
        }
        break;
    case VIRTIO_BLK_T_GET_ID: {
        char s[20] = "hvisor-virblk";
        strncpy(iov[1].iov_base, s, MIN(sizeof(s), iov[1].iov_len));
        break;
    }
    default:
        log_fatal("Operation is not supported");
        err = EOPNOTSUPP;
        break;
    }
    complete_block_operation(dev, req, vq, err, written_len);
}

// Every virtio-blk has a blkproc_thread that is used for reading and writing.
static void *blkproc_thread(void *arg) {
    VirtIODevice *vdev = arg;
    BlkDev *dev = vdev->dev;
    struct blkp_req *breq;
    // get_breq will access the critical section, so lock it.
    pthread_mutex_lock(&dev->mtx);

    for (;;) {
        while (get_breq(dev, &breq)) {
            // blk_proc don't access the critical section, so unlock.
            pthread_mutex_unlock(&dev->mtx);
            blkproc(dev, breq, vdev->vqs);
            pthread_mutex_lock(&dev->mtx);
        }

        if (dev->close) {
            pthread_mutex_unlock(&dev->mtx);
            break;
        }
        pthread_cond_wait(&dev->cond, &dev->mtx);
    }
    pthread_exit(NULL);
    return NULL;
}

// create blk dev.
BlkDev *init_blk_dev(VirtIODevice *vdev) {
    BlkDev *dev = malloc(sizeof(BlkDev));
    vdev->dev = dev;
    dev->config.capacity = -1;
    dev->config.size_max = -1;
    dev->config.seg_max = BLK_SEG_MAX;
    dev->img_fd = -1;
    dev->close = 0;
    // TODO: chang to thread poll
    pthread_mutex_init(&dev->mtx, NULL);
    pthread_cond_init(&dev->cond, NULL);
    TAILQ_INIT(&dev->procq);
    pthread_create(&dev->tid, NULL, blkproc_thread, vdev);
    return dev;
}

int virtio_blk_init(VirtIODevice *vdev, const char *img_path) {
    int img_fd = open(img_path, O_RDWR);
    BlkDev *dev = vdev->dev;
    struct stat st;
    uint64_t blk_size;
    if (img_fd == -1) {
        log_error("cannot open %s, Error code is %d", img_path, errno);
        close(img_fd);
        return -1;
    }
    if (fstat(img_fd, &st) == -1) {
        log_error("cannot stat %s, Error code is %d", img_path, errno);
        close(img_fd);
        return -1;
    }
    blk_size = st.st_size / 512; // 512 bytes per block
    dev->config.capacity = blk_size;
    dev->config.size_max = blk_size;
    dev->img_fd = img_fd;
    vdev->virtio_close = virtio_blk_close;
    log_info("debug: virtio_blk_init: %s, size is %lld", img_path,
             dev->config.capacity);
    return 0;
}

// handle one descriptor list
static struct blkp_req *virtq_blk_handle_one_request(VirtQueue *vq) {
    log_debug("virtq_blk_handle_one_request enter");
    struct blkp_req *breq;
    struct iovec *iov = NULL;
    uint16_t *flags;
    int i, n;
    BlkReqHead *hdr;
    breq = malloc(sizeof(struct blkp_req));
    n = process_descriptor_chain(vq, &breq->idx, &iov, &flags, 0, true);
    breq->iov = iov;
    if (n < 2 || n > BLK_SEG_MAX + 2) {
        log_error("iov's num is wrong, n is %d", n);
        goto err_out;
    }

    if ((flags[0] & VRING_DESC_F_WRITE) != 0) {
        log_error("virt queue's desc chain header should not be writable!");
        goto err_out;
    }

    if (iov[0].iov_len != sizeof(BlkReqHead)) {
        log_error("the size of blk header is %d, it should be %d!",
                  iov[0].iov_len, sizeof(BlkReqHead));
        goto err_out;
    }

    if (iov[n - 1].iov_len != 1 || ((flags[n - 1] & VRING_DESC_F_WRITE) == 0)) {
        log_error(
            "status iov is invalid!, status len is %d, flag is %d, n is %d",
            iov[n - 1].iov_len, flags[n - 1], n);
        goto err_out;
    }

    hdr = (BlkReqHead *)(iov[0].iov_base);
    uint64_t offset = hdr->sector * SECTOR_BSIZE;
    breq->type = hdr->type;
    breq->iovcnt = n;
    breq->offset = offset;

    for (i = 1; i < n - 1; i++)
        if (((flags[i] & VRING_DESC_F_WRITE) == 0) !=
            (breq->type == VIRTIO_BLK_T_OUT)) {
            log_error("flag is conflict with operation");
            goto err_out;
        }

    free(flags);
    return breq;

err_out:
    free(flags);
    free(iov);
    free(breq);
    return NULL;
}

int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("virtio blk notify handler enter");
    BlkDev *blkDev = (BlkDev *)vdev->dev;
    struct blkp_req *breq;
    TAILQ_HEAD(, blkp_req) procq;
    TAILQ_INIT(&procq);
    while (!virtqueue_is_empty(vq)) {
        virtqueue_disable_notify(vq);
        while (!virtqueue_is_empty(vq)) {
            breq = virtq_blk_handle_one_request(vq);
            TAILQ_INSERT_TAIL(&procq, breq, link);
        }
        virtqueue_enable_notify(vq);
    }
    if (TAILQ_EMPTY(&procq)) {
        log_debug("virtio blk notify handler exit, procq is empty");
        return 0;
    }
    pthread_mutex_lock(&blkDev->mtx);
    TAILQ_CONCAT(&blkDev->procq, &procq, link);
    pthread_cond_signal(&blkDev->cond);
    pthread_mutex_unlock(&blkDev->mtx);
    return 0;
}

void virtio_blk_close(VirtIODevice *vdev) {
    BlkDev *dev = vdev->dev;
    pthread_mutex_lock(&dev->mtx);
    dev->close = 1;
    pthread_cond_signal(&dev->cond);
    pthread_mutex_unlock(&dev->mtx);
    pthread_join(dev->tid, NULL);
    pthread_mutex_destroy(&dev->mtx);
    pthread_cond_destroy(&dev->cond);
    close(dev->img_fd);
    free(dev);
    free(vdev->vqs);
    free(vdev);
}