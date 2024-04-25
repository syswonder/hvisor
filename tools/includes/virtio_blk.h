#ifndef _HVISOR_VIRTIO_BLK_H
#define _HVISOR_VIRTIO_BLK_H
#include <stdint.h>
#include <pthread.h>
#include <sys/queue.h>
#include <linux/virtio_blk.h>
#include "virtio.h"

/// Maximum number of segments in a request.
#define BLK_SEG_MAX 256
#define VIRTQUEUE_BLK_MAX_SIZE 512
// A blk sector size
#define SECTOR_BSIZE 512

#define BLK_SUPPORTED_FEATURES ( (1ULL << VIRTIO_BLK_F_SEG_MAX) | (1ULL << VIRTIO_BLK_F_SIZE_MAX) | (1ULL << VIRTIO_F_VERSION_1))

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

BlkDev *init_blk_dev(VirtIODevice *vdev, uint64_t bsize, int img_fd);
int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
#endif /* _HVISOR_VIRTIO_BLK_H */
