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
#include <limits.h>
#include <linux/virtio_blk.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>

// Maximum number of descriptors the backend accepts for one request, and
// the virtqueue size. The common request layout is header + data + status;
// this backend assumes the Linux layout (a 16-byte device-readable header
// descriptor and a 1-byte device-writable status descriptor), so a request
// with N data segments uses N + 2 descriptors. The same limit sizes
// out_buf/in_buf, into which direct and indirect chains are flattened.
//
// Virtio 1.4, 2.7 Split Virtqueues: the queue size is the maximum possible
// descriptor chain length.
// https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html#x1-350007
#define VIRTQUEUE_BLK_MAX_SIZE 512

// Maximum number of data segments in one request; the header and status
// byte are separate descriptors and are not counted by seg_max.
//
// Virtio 1.4, 2.7.5 The Virtqueue Descriptor Table: "Most common is to
// begin the data with a header ... and postfix it with a status tailer."
// https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html#x1-430005
// Virtio 1.4, 5.2.3 Feature bits: VIRTIO_BLK_F_SEG_MAX (2) "Maximum number
// of segments in a request is in seg_max."
// https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html#x1-3850003
// Virtio 1.4, 5.2.4 Device configuration layout: the seg_max field.
// https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html#x1-387004r4
#define BLK_SEG_MAX 510

_Static_assert(BLK_SEG_MAX + 2 <= VIRTQUEUE_BLK_MAX_SIZE,
               "header + data + status must fit the backend iovec arrays");

// A blk sector size
#define SECTOR_BSIZE 512

// VIRTIO_RING_F_EVENT_IDX is also supported, but disabled for now.
#define BLK_SUPPORTED_FEATURES                                                 \
    ((1ULL << VIRTIO_BLK_F_SEG_MAX) | (1ULL << VIRTIO_BLK_F_SIZE_MAX) |        \
     (1ULL << VIRTIO_BLK_F_FLUSH) | (1ULL << VIRTIO_F_VERSION_1) |             \
     (1ULL << VIRTIO_RING_F_INDIRECT_DESC))

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
    bool mutex_initialized;
    bool cond_initialized;
    bool reset; // Device reset in progress: worker must not touch the vq
    bool worker_paused; // Worker parked in reset wait; vq not touched
    struct iovec out_buf[VIRTQUEUE_BLK_MAX_SIZE];
    struct iovec in_buf[VIRTQUEUE_BLK_MAX_SIZE];
} BlkDev;

struct virtio_blk_init_params {
    char img_path[PATH_MAX];
};

extern const struct virtio_device_ops virtio_blk_ops;
extern const struct virtio_config_ops virtio_blk_config_ops;

#endif /* _HVISOR_VIRTIO_BLK_H */
