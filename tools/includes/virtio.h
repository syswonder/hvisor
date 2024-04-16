#ifndef __HVISOR_VIRTIO_H
#define __HVISOR_VIRTIO_H
#include <stdint.h>
#include <stdbool.h>
#include <sys/uio.h>
#include "hvisor.h"
#define VIRT_QUEUE_SIZE 512

typedef struct VirtMmioRegs {
    uint32_t device_id;
    uint32_t dev_feature_sel;
    uint32_t drv_feature_sel;
    uint32_t queue_sel;
    uint32_t interrupt_status;
    uint32_t interrupt_ack;
    uint32_t status;
    uint32_t generation;
    uint64_t dev_feature;
    uint64_t drv_feature;
} VirtMmioRegs;

typedef enum {
    VirtioTNone,
    VirtioTNet,
    VirtioTBlock
} VirtioDeviceType;

typedef struct VirtqDesc {
    uint64_t addr;
    uint32_t len;
    uint16_t flags;
    uint16_t next;
} VirtqDesc;

typedef struct VirtqAvail {
    uint16_t flags;
    uint16_t idx;
    uint16_t ring[VIRT_QUEUE_SIZE];
} VirtqAvail;

typedef struct VirtqUsedElem {
    uint32_t id;
    uint32_t len;
} VirtqUsedElem;

typedef struct VirtqUsed {
    uint16_t flags;
    uint16_t idx;
    struct VirtqUsedElem ring[VIRT_QUEUE_SIZE];
} VirtqUsed;

struct VirtIODevice;
typedef struct VirtIODevice VirtIODevice;
struct VirtQueue;
typedef struct VirtQueue VirtQueue;

struct VirtQueue {
    VirtIODevice *dev;
    uint64_t vq_idx;
    uint64_t num; // queue size. elements number
    uint32_t queue_num_max;

    uint64_t desc_table_addr;
    uint64_t avail_addr;
    uint64_t used_addr;

    volatile VirtqDesc *desc_table; // volatile tells compiler don't optimize it. 
    volatile VirtqAvail *avail_ring;
    volatile VirtqUsed *used_ring;
    int (*notify_handler)(VirtIODevice *vdev, VirtQueue *vq);

    uint16_t last_avail_idx;
    uint16_t last_used_idx;
    uint16_t used_flags;

    uint8_t ready;

	pthread_mutex_t used_ring_lock;
};
// The highest representations of virtio device
struct VirtIODevice
{
    uint32_t id;
    uint32_t vqs_len;
    uint32_t zone_id;
    uint32_t irq_id;
    uint64_t base_addr; // the virtio device's base addr in non root zone's memory
    uint64_t len;       // mmio region's length
    VirtioDeviceType type;
    VirtMmioRegs regs;
    VirtQueue *vqs;
    void *dev;          // according to device type, blk is BlkDev, net is NetDev.
    bool activated;
};


#define VIRT_MAGIC 0x74726976 /* 'virt' */
#define VIRT_VERSION 2
#define VIRT_VENDOR 0x48564953 /* 'HVIS' */

/* v1.0 compliant */
#define VIRTIO_F_VERSION_1 ((uint64_t)1 << 32)

/*
 * Control registers
 */

/* Magic value ("virt" string) - Read Only */
#define VIRTIO_MMIO_MAGIC_VALUE		0x000

/* Virtio device version - Read Only */
#define VIRTIO_MMIO_VERSION		0x004

/* Virtio device ID - Read Only */
#define VIRTIO_MMIO_DEVICE_ID		0x008

/* Virtio vendor ID - Read Only */
#define VIRTIO_MMIO_VENDOR_ID		0x00c

/* Bitmask of the features supported by the device (host)
 * (32 bits per set) - Read Only */
#define VIRTIO_MMIO_DEVICE_FEATURES	0x010

/* Device (host) features set selector - Write Only */
#define VIRTIO_MMIO_DEVICE_FEATURES_SEL	0x014

/* Bitmask of features activated by the driver (guest)
 * (32 bits per set) - Write Only */
#define VIRTIO_MMIO_DRIVER_FEATURES	0x020

/* Activated features set selector - Write Only */
#define VIRTIO_MMIO_DRIVER_FEATURES_SEL	0x024

/* Queue selector - Write Only */
#define VIRTIO_MMIO_QUEUE_SEL		0x030

/* Maximum size of the currently selected queue - Read Only */
#define VIRTIO_MMIO_QUEUE_NUM_MAX	0x034

/* Queue size for the currently selected queue - Write Only */
#define VIRTIO_MMIO_QUEUE_NUM		0x038

/* Ready bit for the currently selected queue - Read Write */
#define VIRTIO_MMIO_QUEUE_READY		0x044

/* Queue notifier - Write Only */
#define VIRTIO_MMIO_QUEUE_NOTIFY	0x050

/* Interrupt status - Read Only */
#define VIRTIO_MMIO_INTERRUPT_STATUS	0x060

/* Interrupt acknowledge - Write Only */
#define VIRTIO_MMIO_INTERRUPT_ACK	0x064

/* Device status register - Read Write */
#define VIRTIO_MMIO_STATUS		0x070

/* Selected queue's Descriptor Table address, 64 bits in two halves */
#define VIRTIO_MMIO_QUEUE_DESC_LOW	0x080
#define VIRTIO_MMIO_QUEUE_DESC_HIGH	0x084

/* Selected queue's Available Ring address, 64 bits in two halves */
#define VIRTIO_MMIO_QUEUE_AVAIL_LOW	0x090
#define VIRTIO_MMIO_QUEUE_AVAIL_HIGH	0x094

/* Selected queue's Used Ring address, 64 bits in two halves */
#define VIRTIO_MMIO_QUEUE_USED_LOW	0x0a0
#define VIRTIO_MMIO_QUEUE_USED_HIGH	0x0a4

/* Shared memory region id */
#define VIRTIO_MMIO_SHM_SEL             0x0ac

/* Shared memory region length, 64 bits in two halves */
#define VIRTIO_MMIO_SHM_LEN_LOW         0x0b0
#define VIRTIO_MMIO_SHM_LEN_HIGH        0x0b4

/* Shared memory region base address, 64 bits in two halves */
#define VIRTIO_MMIO_SHM_BASE_LOW        0x0b8
#define VIRTIO_MMIO_SHM_BASE_HIGH       0x0bc

/* Configuration atomicity value */
#define VIRTIO_MMIO_CONFIG_GENERATION	0x0fc

/* The config space is defined by each driver as
 * the per-driver configuration space - Read Write */
#define VIRTIO_MMIO_CONFIG		0x100

/*
 * Interrupt flags (re: interrupt status & acknowledge registers)
 */

#define VIRTIO_MMIO_INT_VRING		(1 << 0)
#define VIRTIO_MMIO_INT_CONFIG		(1 << 1)

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type);

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type);


void virtio_dev_reset(VirtIODevice *vdev);

void virtqueue_reset(VirtQueue *vq, int idx);

bool virtqueue_is_empty(VirtQueue *vq);

uint16_t virtqueue_pop_desc_chain_head(VirtQueue *vq);

void virtqueue_disable_notify(VirtQueue *vq);
void virtqueue_enable_notify(VirtQueue *vq);

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx);
void* get_virt_addr(void *addr);
void* get_phys_addr(void *addr);
int virtio_handle_req(volatile struct device_req *req);
int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                struct iovec **iov, uint16_t **flags, int append_len);
void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen);
void virtio_inject_irq(uint32_t target_zone, uint32_t irq_id);
void try_inject_irq(VirtQueue *vq, int no_more_chains);
void handle_virtio_requests();
int virtio_init();
int virtio_start(int argc, char *argv[]);

/// check circular queue is full. size must be a power of 2
int is_queue_full(unsigned int front, unsigned int rear, unsigned int size);
int is_queue_empty(unsigned int front, unsigned int rear);

/* This marks a buffer as continuing via the next field. */
#define VRING_DESC_F_NEXT	1
/* This marks a buffer as write-only (otherwise read-only). */
#define VRING_DESC_F_WRITE	2
/* This means the buffer contains a list of buffer descriptors. */
#define VRING_DESC_F_INDIRECT	4

/* The Host uses this in used->flags to advise the Guest: don't kick me when
 * you add a buffer.  It's unreliable, so it's simply an optimization.  Guest
 * will still kick if it's out of buffers. */
#define VRING_USED_F_NO_NOTIFY	1
/* The Guest uses this in avail->flags to advise the Host: don't interrupt me
 * when you consume a buffer.  It's unreliable, so it's simply an
 * optimization.  */
#define VRING_AVAIL_F_NO_INTERRUPT	1
#endif /* __HVISOR_VIRTIO_H */

