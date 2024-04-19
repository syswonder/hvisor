#ifndef __HVISOR_VIRTIO_H
#define __HVISOR_VIRTIO_H
#include <stdint.h>
#include <stdbool.h>
#include <sys/uio.h>
#include <linux/virtio_ring.h>
#include <linux/virtio_mmio.h>
#include <linux/virtio_config.h>
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

typedef struct vring_desc VirtqDesc;
typedef struct vring_avail VirtqAvail;
typedef struct vring_used_elem VirtqUsedElem;
typedef struct vring_used VirtqUsed;

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
	uint8_t event_idx_enabled;
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
// used event idx for driver telling device when to notify driver.
#define VQ_USED_EVENT(vq) ((vq)->avail_ring->ring[(vq)->num])
// avail event idx for device telling driver when to notify device.
#define VQ_AVAIL_EVENT(vq) (*(__uint16_t *)&(vq)->used_ring->ring[(vq)->num])

#define VIRT_MAGIC 0x74726976 /* 'virt' */
#define VIRT_VERSION 2
#define VIRT_VENDOR 0x48564953 /* 'HVIS' */

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type);

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type);


void virtio_dev_reset(VirtIODevice *vdev);

void virtqueue_reset(VirtQueue *vq, int idx);

bool virtqueue_is_empty(VirtQueue *vq);

// uint16_t virtqueue_pop_desc_chain_head(VirtQueue *vq);

void virtqueue_disable_notify(VirtQueue *vq);
void virtqueue_enable_notify(VirtQueue *vq);

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx);
void* get_virt_addr(void *addr);
void* get_phys_addr(void *addr);
int virtio_handle_req(volatile struct device_req *req);
int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                struct iovec **iov, uint16_t **flags, int append_len);
void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen);
void virtio_inject_irq(VirtQueue *vq);
void handle_virtio_requests();
int virtio_init();
int virtio_start(int argc, char *argv[]);

/// check circular queue is full. size must be a power of 2
int is_queue_full(unsigned int front, unsigned int rear, unsigned int size);
int is_queue_empty(unsigned int front, unsigned int rear);

#endif /* __HVISOR_VIRTIO_H */

