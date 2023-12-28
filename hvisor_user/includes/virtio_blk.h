#ifndef _HVISOR_VIRTIO_BLK_H
#define _HVISOR_VIRTIO_BLK_H
#include <stdint.h>
typedef uint16_t  __virtio16;
typedef uint32_t  __virtio32;
typedef uint64_t  __virtio64;

struct virtio_blk_config {
	/* The capacity (in 512-byte sectors). */
	__virtio64 capacity;
	/* The maximum segment size (if VIRTIO_BLK_F_SIZE_MAX) */
	__virtio32 size_max;
	/* The maximum number of segments (if VIRTIO_BLK_F_SEG_MAX) */
	__virtio32 seg_max;
	/* geometry of the device (if VIRTIO_BLK_F_GEOMETRY) */
	struct virtio_blk_geometry {
		__virtio16 cylinders;
		uint8_t heads;
		uint8_t sectors;
	} geometry;

	/* block size of device (if VIRTIO_BLK_F_BLK_SIZE) */
	__virtio32 blk_size;

	/* the next 4 entries are guarded by VIRTIO_BLK_F_TOPOLOGY  */
	/* exponent for physical block per logical block. */
	uint8_t physical_block_exp;
	/* alignment offset in logical blocks. */
	uint8_t alignment_offset;
	/* minimum I/O size without performance penalty in logical blocks. */
	__virtio16 min_io_size;
	/* optimal sustained I/O size in logical blocks. */
	__virtio32 opt_io_size;

	/* writeback mode (if VIRTIO_BLK_F_CONFIG_WCE) */
	uint8_t wce;
	uint8_t unused;

	/* number of vqs, only available when VIRTIO_BLK_F_MQ is set */
	__virtio16 num_queues;

	/* the next 3 entries are guarded by VIRTIO_BLK_F_DISCARD */
	/*
	 * The maximum discard sectors (in 512-byte sectors) for
	 * one segment.
	 */
	__virtio32 max_discard_sectors;
	/*
	 * The maximum number of discard segments in a
	 * discard command.
	 */
	__virtio32 max_discard_seg;
	/* Discard commands must be aligned to this number of sectors. */
	__virtio32 discard_sector_alignment;

	/* the next 3 entries are guarded by VIRTIO_BLK_F_WRITE_ZEROES */
	/*
	 * The maximum number of write zeroes sectors (in 512-byte sectors) in
	 * one segment.
	 */
	__virtio32 max_write_zeroes_sectors;
	/*
	 * The maximum number of segments in a write zeroes
	 * command.
	 */
	__virtio32 max_write_zeroes_seg;
	/*
	 * Set if a VIRTIO_BLK_T_WRITE_ZEROES request may result in the
	 * deallocation of one or more of the sectors.
	 */
	uint8_t write_zeroes_may_unmap;

	uint8_t unused1[3];
} __attribute__((packed));


typedef struct virtio_blk_config BlkConfig;
BlkConfig *init_blk_config(uint64_t bsize);

/* Feature bits */
#define VIRTIO_BLK_F_SIZE_MAX	(1<<1)	/* Indicates maximum segment size */
#define VIRTIO_BLK_F_SEG_MAX	(1<<2)	/* Indicates maximum # of segments */

#define BLK_SIZE_MAX 524288 // 128 * 4096
#define BLK_SEG_MAX 512

#define VIRTQUEUE_BLK_MAX_SIZE 256
#endif /* _HVISOR_VIRTIO_BLK_H */
