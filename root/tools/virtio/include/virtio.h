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
#ifndef __HVISOR_VIRTIO_H
#define __HVISOR_VIRTIO_H
#include "hvisor.h"
#include "safe_cjson.h"
#include <linux/virtio_config.h>
#include <linux/virtio_mmio.h>
#include <linux/virtio_ring.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <sys/uio.h>
#include <unistd.h>

#define VIRT_QUEUE_SIZE 512

typedef struct VirtMmioRegs {
    uint32_t device_id;       // Device type id, see VirtioDeviceType
    uint32_t dev_feature_sel; // device_features_selection, the driver frontend
                              // writes this register to obtain a group of
                              // 32-bit device_features
    uint32_t drv_feature_sel; // driver_features_selection, the driver frontend
                              // writes this register to specify a group of
                              // 32-bit activated device_features
    uint32_t queue_sel; // The driver frontend writes this register to select
                        // which virtqueue to use
    uint32_t interrupt_status;
    // interrupt_count doesn't exist in virtio specification,
    // only used for ensuring the correctness of interrupt_status.
    uint32_t interrupt_count;
    uint32_t status;
    uint32_t generation;
    uint64_t dev_feature; // device_features
    uint64_t drv_feature;
} VirtMmioRegs;

typedef enum {
    VirtioTNone,
    VirtioTNet,
    VirtioTBlock,
    VirtioTConsole,
    VirtioTGPU = 16
} VirtioDeviceType;

// Convert VirtioDeviceType to const char *
const char *virtio_device_type_to_string(VirtioDeviceType type);

typedef struct vring_desc VirtqDesc;
typedef struct vring_avail VirtqAvail;
typedef struct vring_used_elem VirtqUsedElem;
typedef struct vring_used VirtqUsed;

struct VirtIODevice;
typedef struct VirtIODevice VirtIODevice;
struct VirtQueue;
typedef struct VirtQueue VirtQueue;

struct VirtQueue {
    VirtIODevice *dev; // The device which the virtqueue belongs to
    uint64_t vq_idx;   // Index of the virtqueue
    uint64_t
        num; // The capacity of the virtqueue determined by the driver frontend
    uint32_t queue_num_max; // The maximum capacity of the virtqueue, informed
                            // by the backend

    uint64_t desc_table_addr; // Descriptor table address (physical address set
                              // by zonex)
    uint64_t
        avail_addr; // Available ring address (physical address set by zonex)
    uint64_t used_addr; // Used ring address (physical address set by zonex)

    // Obtained by get_virt_addr
    volatile VirtqDesc
        *desc_table; // Descriptor table (physical address set by zone0)
    volatile VirtqAvail
        *avail_ring; // Available ring (physical address set by zone0)
    volatile VirtqUsed *used_ring; // Used ring (physical address set by zone0)
    int (*notify_handler)(
        VirtIODevice *vdev,
        VirtQueue *vq); // Called when the virtqueue has requests to process

    uint16_t last_avail_idx; // The tail idx of the available ring during the
                             // last processing (tail/newest request idx)
    uint16_t last_used_idx;  // The tail idx of the used ring during the last
                             // processing (tail/newest response idx)

    uint8_t ready;             // Whether it is ready
    uint8_t event_idx_enabled; // Whether the VIRTIO_RING_F_EVENT_IDX feature is
                               // enabled This feature allows the device to
                               // record the current position of the element on
                               // the Avail Ring being processed in the last
                               // element of the Used Ring, thereby informing
                               // the frontend driver of the backend processing
                               // progress Enabling this feature will change the
                               // flags field of the avail_ring
    pthread_mutex_t used_ring_lock; // Used ring lock
};

// The highest abstruct representations of virtio device
struct VirtIODevice {
    uint32_t vqs_len; // Number of virtqueues
    uint32_t zone_id; // ID of the zone to which it belongs, initialized in
                      // create_virtio_device
    uint32_t irq_id; // Device interrupt ID, initialized in create_virtio_device
    uint64_t base_addr; // The virtio device's base address in non-root zone's
                        // memory, initialized in create_virtio_device
    uint64_t
        len; // Length of the mmio region, initialized in create_virtio_device
    VirtioDeviceType type; // Device type, initialized in create_virtio_device
    VirtMmioRegs regs;     // MMIO registers of the device
                           // Initialized in create_virtio_device, subsequently
                           // negotiated by the driver frontend
    VirtQueue *vqs;        // Array of virtqueues of the device, initialized in
                           // init_virtio_queue
    void
        *dev; // According to device type, blk is BlkDev, net is NetDev, console
              // is ConsoleDev Pointer to the specific device's special config
    void (*virtio_close)(
        VirtIODevice *vdev); // Function called when closing the virtio device
    bool activated;          // Whether the current virtio device is activated
};

// used event idx for driver telling device when to notify driver.
#define VQ_USED_EVENT(vq) ((vq)->avail_ring->ring[(vq)->num])
// avail event idx for device telling driver when to notify device.
#define VQ_AVAIL_EVENT(vq) (*(uint16_t *)&(vq)->used_ring->ring[(vq)->num])

#define VIRT_MAGIC 0x74726976 /* 'virt' */

#define VIRT_VERSION 2

#define VIRT_VENDOR 0x48564953 /* 'HVIS' */

// Set net and console to non-blocking
int set_nonblocking(int fd);

int get_zone_ram_index(void *zonex_ipa, int zone_id);

/// Check if circular queue is full. size must be a power of 2
int is_queue_full(unsigned int front, unsigned int rear, unsigned int size);

int is_queue_empty(unsigned int front, unsigned int rear);

void write_barrier(void);

void read_barrier(void);

void rw_barrier(void);

VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t zone_id,
                                   uint64_t base_addr, uint64_t len,
                                   uint32_t irq_id, void *arg0, void *arg1);

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type);

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type);

void virtio_dev_reset(VirtIODevice *vdev);

void virtqueue_reset(VirtQueue *vq, int idx);

bool virtqueue_is_empty(VirtQueue *vq);

// uint16_t virtqueue_pop_desc_chain_head(VirtQueue *vq);

void virtqueue_disable_notify(VirtQueue *vq);

void virtqueue_enable_notify(VirtQueue *vq);

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx);

void *get_virt_addr(void *zonex_ipa, int zone_id);

void virtqueue_set_avail(VirtQueue *vq);

void virtqueue_set_used(VirtQueue *vq);

int descriptor2iov(int i, volatile VirtqDesc *vd, struct iovec *iov,
                   uint16_t *flags, int zone_id, bool copy_flags);

int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                             struct iovec **iov, uint16_t **flags,
                             int append_len, bool copy_flags);

void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen);

uint64_t virtio_mmio_read(VirtIODevice *vdev, uint64_t offset, unsigned size);

void virtio_mmio_write(VirtIODevice *vdev, uint64_t offset, uint64_t value,
                       unsigned size);

bool in_range(uint64_t value, uint64_t lower, uint64_t len);

void virtio_inject_irq(VirtQueue *vq); // unused

void virtio_finish_cfg_req(uint32_t target_cpu, uint64_t value);

int virtio_handle_req(volatile struct device_req *req);

void virtio_close();

void handle_virtio_requests();

void initialize_log();

int virtio_init();

int create_virtio_device_from_json(cJSON *device_json, int zone_id);

int virtio_start_from_json(char *json_path);

int virtio_start(int argc, char *argv[]);

void *read_file(char *filename, uint64_t *filesize);

#endif /* __HVISOR_VIRTIO_H */
