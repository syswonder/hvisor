#ifndef _HVISOR_VIRTIO_BLK_H
#define _HVISOR_VIRTIO_BLK_H
#include <stdint.h>
#include <pthread.h>
#include <sys/queue.h>
#include "virtio.h"

/* Feature bits */
#define VIRTIO_BLK_F_SIZE_MAX	(1<<1)	/* Indicates maximum segment size */
#define VIRTIO_BLK_F_SEG_MAX	(1<<2)	/* Indicates maximum # of segments */


#define BLK_SIZE_MAX 122880 // indicate how many of 512B
#define BLK_SEG_MAX 256

#define VIRTQUEUE_BLK_MAX_SIZE 512

#define VIRTIO_BLK_T_IN		0
#define VIRTIO_BLK_T_OUT	1
/* Cache flush command */
#define VIRTIO_BLK_T_FLUSH	4

/* Get device ID command */
#define VIRTIO_BLK_T_GET_ID    8

// A blk sector size
#define SECTOR_BSIZE 512

#define VIRTIO_BLK_S_OK         0
#define VIRTIO_BLK_S_IOERR      1
#define VIRTIO_BLK_S_UNSUPP     2

struct virtio_blk_req_head {
	uint32_t req_type;
	uint32_t reserved;
	uint64_t sector;
};

struct virtio_blk_config {
	/* The capacity (in 512-byte sectors). */
	uint64_t capacity;
	/* The maximum segment size (if VIRTIO_BLK_F_SIZE_MAX) */
	uint32_t size_max;
	/* The maximum number of segments (if VIRTIO_BLK_F_SEG_MAX) */
	uint32_t seg_max;
	/* geometry of the device (if VIRTIO_BLK_F_GEOMETRY) */
	struct virtio_blk_geometry {
		uint16_t cylinders;
		uint8_t heads;
		uint8_t sectors;
	} geometry;

	/* block size of device (if VIRTIO_BLK_F_BLK_SIZE) */
	uint32_t blk_size;

	/* the next 4 entries are guarded by VIRTIO_BLK_F_TOPOLOGY  */
	/* exponent for physical block per logical block. */
	uint8_t physical_block_exp;
	/* alignment offset in logical blocks. */
	uint8_t alignment_offset;
	/* minimum I/O size without performance penalty in logical blocks. */
	uint16_t min_io_size;
	/* optimal sustained I/O size in logical blocks. */
	uint32_t opt_io_size;

	/* writeback mode (if VIRTIO_BLK_F_CONFIG_WCE) */
	uint8_t wce;
	uint8_t unused;

	/* number of vqs, only available when VIRTIO_BLK_F_MQ is set */
	uint16_t num_queues;

	/* the next 3 entries are guarded by VIRTIO_BLK_F_DISCARD */
	/*
	 * The maximum discard sectors (in 512-byte sectors) for
	 * one segment.
	 */
	uint32_t max_discard_sectors;
	/*
	 * The maximum number of discard segments in a
	 * discard command.
	 */
	uint32_t max_discard_seg;
	/* Discard commands must be aligned to this number of sectors. */
	uint32_t discard_sector_alignment;

	/* the next 3 entries are guarded by VIRTIO_BLK_F_WRITE_ZEROES */
	/*
	 * The maximum number of write zeroes sectors (in 512-byte sectors) in
	 * one segment.
	 */
	uint32_t max_write_zeroes_sectors;
	/*
	 * The maximum number of segments in a write zeroes
	 * command.
	 */
	uint32_t max_write_zeroes_seg;
	/*
	 * Set if a VIRTIO_BLK_T_WRITE_ZEROES request may result in the
	 * deallocation of one or more of the sectors.
	 */
	uint8_t write_zeroes_may_unmap;

	uint8_t unused1[3];
} __attribute__((packed));

typedef struct virtio_blk_config BlkConfig;
typedef struct virtio_blk_req_head BlkReqHead;

enum blkop {
	BLK_READ,
	BLK_WRITE
};
// A request needed to process by blk thread.
struct blkp_req {
	TAILQ_ENTRY(blkp_req) link;
    struct iovec iov[BLK_SEG_MAX+2];
	int iovcnt;
	uint64_t offset;
	uint16_t idx;
	enum blkop type; 
};

typedef struct virtio_blk_dev {
    BlkConfig config;
    int img_fd;
	// describe the thread executes read and write.
	pthread_t tid;
	pthread_mutex_t mtx;
	pthread_cond_t cond;
	TAILQ_HEAD(, blkp_req) procq;
	int closing;
} BlkDev;

BlkDev *init_blk_dev(VirtIODevice *vdev, uint64_t bsize, int img_fd);
int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
#endif /* _HVISOR_VIRTIO_BLK_H */
