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
#include <stdbool.h>
#include <stdint.h>

/// Maximum number of segments in a request.
#define BLK_SEG_MAX 512
#define VIRTQUEUE_BLK_MAX_SIZE 512
// A blk sector size
#define SECTOR_BSIZE 512

// VIRTIO_RING_F_INDIRECT_DESC and VIRTIO_RING_F_EVENT_IDX are also supported,
// for some reason we disable them for now.
#define BLK_SUPPORTED_FEATURES                                                 \
    ((1ULL << VIRTIO_BLK_F_SEG_MAX) | (1ULL << VIRTIO_BLK_F_SIZE_MAX) |        \
     (1ULL << VIRTIO_BLK_F_FLUSH) | (1ULL << VIRTIO_F_VERSION_1))

typedef struct virtio_blk_config BlkConfig;
typedef struct virtio_blk_outhdr BlkReqHead;

typedef struct virtio_blk_dev {
    BlkConfig config;
    int img_fd;
    pthread_t tid;
    pthread_mutex_t mtx;
    pthread_cond_t cond;
    bool close;
    bool thread_started;
    bool reset; // Device reset in progress: worker must not touch the vq
    bool worker_paused; // Worker parked in reset wait; vq not touched
    struct iovec out_buf[VIRTQUEUE_BLK_MAX_SIZE];
    struct iovec in_buf[VIRTQUEUE_BLK_MAX_SIZE];
} BlkDev;

struct virtio_blk_init_params {
    const char *img_path;
};

extern const struct virtio_device_ops virtio_blk_ops;
extern const struct virtio_config_ops virtio_blk_config_ops;

#endif /* _HVISOR_VIRTIO_BLK_H */
