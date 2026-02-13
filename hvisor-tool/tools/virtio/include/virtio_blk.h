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
#ifndef _HVISOR_VIRTIO_BLK_H
#define _HVISOR_VIRTIO_BLK_H
#include "virtio.h"
#include <linux/virtio_blk.h>
#include <pthread.h>
#include <stdint.h>
#include <sys/queue.h>

/// Maximum number of segments in a request.
#define BLK_SEG_MAX 512
#define VIRTQUEUE_BLK_MAX_SIZE 512
// A blk sector size
#define SECTOR_BSIZE 512

// VIRTIO_RING_F_INDIRECT_DESC and VIRTIO_RING_F_EVENT_IDX are also supported,
// for some reason we disable them for now.
#define BLK_SUPPORTED_FEATURES                                                 \
    ((1ULL << VIRTIO_BLK_F_SEG_MAX) | (1ULL << VIRTIO_BLK_F_SIZE_MAX) |        \
     (1ULL << VIRTIO_F_VERSION_1))

typedef struct virtio_blk_config BlkConfig;
typedef struct virtio_blk_outhdr BlkReqHead;

// A request needed to process by blk thread.
struct blkp_req {
    TAILQ_ENTRY(blkp_req) link;
    struct iovec *iov;
    int iovcnt;
    uint64_t offset;
    uint32_t type;
    uint16_t idx;
};

typedef struct virtio_blk_dev {
    BlkConfig config;
    int img_fd;
    // describe the worker thread that executes read, write and ioctl.
    pthread_t tid;
    pthread_mutex_t mtx;
    pthread_cond_t cond;
    TAILQ_HEAD(, blkp_req) procq;
    int close;
} BlkDev;

BlkDev *init_blk_dev(VirtIODevice *vdev);
int virtio_blk_init(VirtIODevice *vdev, const char *img_path);
int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
void virtio_blk_close(VirtIODevice *vdev);

#endif /* _HVISOR_VIRTIO_BLK_H */
