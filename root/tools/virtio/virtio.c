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
#include <errno.h>
#include <fcntl.h>
#include <getopt.h>
#include <inttypes.h>
#include <limits.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/prctl.h>
#include <sys/signalfd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/uio.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

#include "hvisor.h"
#include "json_parse.h"
#include "loader.h"
#include "log.h"
#include "safe_cjson.h"
#include "virtio.h"
#include "virtio_blk.h"
#include "virtio_console.h"
#include "virtio_net.h"
#ifdef ENABLE_VIRTIO_GPU
#include "virtio_gpu.h"
#endif
#include "virtio_scmi.h"

/// hvisor kernel module fd
int ko_fd;
static int efd = -1;
static int sfd = -1;
static int epoll_fd = -1;
volatile struct virtio_bridge *virtio_bridge;

pthread_mutex_t RES_MUTEX = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t VDEV_MUTEX = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t ZONE_MEM_MUTEX = PTHREAD_MUTEX_INITIALIZER;
VirtIODevice *vdevs[MAX_DEVS];
int vdevs_num;

#define VIRTIO_CTRL_SOCKET_PATH "/run/hvisor-virtio.sock"
#define VIRTIO_CTRL_OP_ADD "add"
#define VIRTIO_CTRL_MSG_LEN 256

typedef struct virtio_control_request {
    char op[16];
    char json_path[PATH_MAX];
} VirtioControlRequest;

typedef struct virtio_control_response {
    int status;
    char message[VIRTIO_CTRL_MSG_LEN];
} VirtioControlResponse;

static int ctrl_fd = -1;
static pthread_t ctrl_tid;
static atomic_bool ctrl_running = false;
static bool ctrl_thread_started;
static pthread_mutex_t ctrl_client_lock = PTHREAD_MUTEX_INITIALIZER;
static int ctrl_client_fd = -1;
static _Atomic uint64_t virtio_irq_trace_seq;

static void stop_virtio_control_server(void);

static bool virtio_trace_sample(uint64_t seq) {
    return seq < 128 || (seq != 0 && (seq & (seq - 1)) == 0);
}

static int virtio_deassert_line_locked(VirtIODevice *vdev) {
#ifdef LOONGARCH64
    struct hvisor_irq_line_args args = {
        .zone_id = vdev->zone_id,
        .irq_id = vdev->irq_id,
    };
    int ret;
    do {
        ret = ioctl(ko_fd, HVISOR_DEASSERT_IRQ, &args);
    } while (ret < 0 && errno == EINTR);
    if (ret < 0) {
        log_error("deassert failed: zone=%u irq=%u errno=%d (%s)",
                  vdev->zone_id, vdev->irq_id, errno, strerror(errno));
    }
    return ret;
#else
    (void)vdev;
    return 0;
#endif
}

struct zone_mem_region {
    uintptr_t virt_addr;
    uintptr_t zone0_ipa;
    uintptr_t zonex_ipa;
    uintptr_t mem_size;
};

struct zone_mem {
    struct zone_mem_region regions[CONFIG_MAX_MEMORY_REGIONS];
    size_t num_regions;
};

struct zone_mem zone_mem[MAX_ZONES];

typedef struct virtio_memory_region_config {
    uint64_t zone0_ipa;
    uint64_t zonex_ipa;
    uint64_t size;
} VirtioMemoryRegionConfig;

typedef struct virtio_device_config {
    VirtioDeviceType type;
    uint32_t zone_id;
    uint64_t addr;
    uint64_t len;
    uint32_t irq;
    bool enabled;
    const struct virtio_config_ops *config_ops;
    void *params;
} VirtioDeviceConfig;

typedef struct virtio_zone_config {
    uint32_t zone_id;
    size_t memory_region_num;
    VirtioMemoryRegionConfig memory_regions[CONFIG_MAX_MEMORY_REGIONS];
    size_t device_num;
    VirtioDeviceConfig devices[MAX_DEVS];
} VirtioZoneConfig;

typedef struct virtio_config {
    size_t zone_num;
    VirtioZoneConfig zones[MAX_ZONES];
} VirtioConfig;

const char *virtio_device_type_to_string(VirtioDeviceType type) {
    switch (type) {
    case VirtioTNone:
        return "virtio-none";
    case VirtioTNet:
        return "virtio-net";
    case VirtioTBlock:
        return "virtio-blk";
    case VirtioTConsole:
        return "virtio-console";
    case VirtioTSCMI:
        return "virtio-scmi";
    case VirtioTGPU:
        return "virtio-gpu";
    default:
        return "unknown";
    }
}

int set_nonblocking(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags == -1) {
        log_error("fcntl(F_GETFL) failed");
        return -1;
    }
    if (fcntl(fd, F_SETFL, flags | O_NONBLOCK) == -1) {
        log_error("fcntl(F_SETFL) failed");
        return -1;
    }
    return 0;
}

inline int is_queue_full(unsigned int front, unsigned int rear,
                         unsigned int size) {
    if (((rear + 1) & (size - 1)) == front) {
        return 1;
    } else {
        return 0;
    }
}

/// Write barrier to make sure all write operations are finished before this
/// operation
inline void write_barrier(void) {
#ifdef ARM64
    asm volatile("dmb ishst" ::: "memory");
#endif
#ifdef RISCV64
    asm volatile("fence w,w" ::: "memory");
#endif
#ifdef LOONGARCH64
    asm volatile("dbar 0" ::: "memory");
#endif
#ifdef X86_64
    asm volatile("" ::: "memory");
#endif
}

inline void read_barrier(void) {
#ifdef ARM64
    asm volatile("dmb ishld" ::: "memory");
#endif
#ifdef RISCV64
    asm volatile("fence r,r" ::: "memory");
#endif
#ifdef LOONGARCH64
    asm volatile("dbar 0" ::: "memory");
#endif
#ifdef X86_64
    asm volatile("" ::: "memory");
#endif
}

inline void rw_barrier(void) {
#ifdef ARM64
    asm volatile("dmb ish" ::: "memory");
#endif
#ifdef RISCV64
    asm volatile("fence rw,rw" ::: "memory");
#endif
#ifdef LOONGARCH64
    asm volatile("dbar 0" ::: "memory");
#endif
#ifdef X86_64
    asm volatile("" ::: "memory");
#endif
}

// ---------------------------------------------------------------------------
// Device ops table — one pointer per device type, defined in each device's .c
static const struct virtio_device_ops *const device_ops_table[] = {
    [VirtioTBlock] = &virtio_blk_ops,       [VirtioTNet] = &virtio_net_ops,
    [VirtioTConsole] = &virtio_console_ops, [VirtioTSCMI] = &virtio_scmi_ops,
#ifdef ENABLE_VIRTIO_GPU
    [VirtioTGPU] = &virtio_gpu_ops,
#endif
};

static const struct virtio_device_ops *lookup_ops(VirtioDeviceType type) {
    int n = (int)(sizeof(device_ops_table) / sizeof(device_ops_table[0]));
    if (type <= VirtioTNone || (int)type >= n)
        return NULL;
    return device_ops_table[type];
}

static const struct virtio_config_ops *const config_ops_table[] = {
    [VirtioTBlock] = &virtio_blk_config_ops,
    [VirtioTNet] = &virtio_net_config_ops,
    [VirtioTConsole] = &virtio_console_config_ops,
    [VirtioTSCMI] = &virtio_scmi_config_ops,
#ifdef ENABLE_VIRTIO_GPU
    [VirtioTGPU] = &virtio_gpu_config_ops,
#endif
};

static const struct virtio_config_ops *
lookup_config_ops(VirtioDeviceType type) {
    int n = (int)(sizeof(config_ops_table) / sizeof(config_ops_table[0]));
    if (type <= VirtioTNone || (int)type >= n)
        return NULL;
    return config_ops_table[type];
}

static int init_virtio_queue(VirtIODevice *vdev,
                             const struct virtio_device_ops *ops);

// ---------------------------------------------------------------------------
// Device creation — fully table-driven.
// ---------------------------------------------------------------------------

static void destroy_unpublished_virtio_device(VirtIODevice *vdev) {
    if (!vdev)
        return;

    if (vdev->virtio_close) {
        vdev->virtio_close(vdev);
        return;
    }

    free(vdev->vqs);
    free(vdev);
}

static int publish_virtio_devices(VirtIODevice **new_devs, size_t count) {
    if (count == 0)
        return 0;

    pthread_mutex_lock(&VDEV_MUTEX);
    if (count > MAX_DEVS - (size_t)vdevs_num) {
        pthread_mutex_unlock(&VDEV_MUTEX);
        log_error("virtio device num exceed max limit");
        return -1;
    }

    for (size_t i = 0; i < count; i++)
        vdevs[vdevs_num++] = new_devs[i];
    pthread_mutex_unlock(&VDEV_MUTEX);
    return 0;
}

static VirtIODevice *
create_virtio_device_unpublished(VirtioDeviceType dev_type, uint32_t zone_id,
                                 uint64_t base_addr, uint64_t len,
                                 uint32_t irq_id, const void *params) {
    const struct virtio_device_ops *ops = lookup_ops(dev_type);
    if (!ops) {
        log_error("unsupported virtio device type %d", dev_type);
        return NULL;
    }

    log_info(
        "create virtio device type %s, zone id %d, base addr %lx, len %lx, "
        "irq id %d",
        virtio_device_type_to_string(dev_type), zone_id, base_addr, len,
        irq_id);

    VirtIODevice *vdev = calloc(1, sizeof(VirtIODevice));
    if (!vdev) {
        log_error("failed to allocate virtio device");
        return NULL;
    }

    init_mmio_regs(&vdev->regs, dev_type);
    vdev->base_addr = base_addr;
    vdev->len = len;
    vdev->zone_id = zone_id;
    vdev->irq_id = irq_id;
    vdev->type = dev_type;
    pthread_mutex_init(&vdev->interrupt_lock, NULL);
    vdev->interrupt_line_asserted = false;
    vdev->regs.dev_feature = ops->features;
    vdev->virtio_close = ops->close;
    vdev->status_changed = ops->status_changed;

    // Allocate virtqueues before device init: net/console register their
    // fds with the already-running event-monitor epoll inside ops->init,
    // and the event handlers dereference vdev->vqs.  The pre-ops-table
    // code also initialized queues first — keep that ordering.
    if (init_virtio_queue(vdev, ops) != 0)
        goto err;

    if (ops->init(vdev, params) != 0)
        goto err;

    log_info("create %s success", virtio_device_type_to_string(dev_type));
    return vdev;

err:
    ops->close(vdev);
    return NULL;
}

// Create and publish one device for the legacy virtio start path.
VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t zone_id,
                                   uint64_t base_addr, uint64_t len,
                                   uint32_t irq_id, const void *params) {
    VirtIODevice *vdev = create_virtio_device_unpublished(
        dev_type, zone_id, base_addr, len, irq_id, params);
    if (!vdev)
        return NULL;

    if (publish_virtio_devices(&vdev, 1) != 0) {
        destroy_unpublished_virtio_device(vdev);
        return NULL;
    }
    return vdev;
}

static int init_virtio_queue(VirtIODevice *vdev,
                             const struct virtio_device_ops *ops) {
    log_info("Initializing virtio queue for zone:%d, device type:%s",
             vdev->zone_id, virtio_device_type_to_string(ops->type));

    if (ops->num_queues == 0 || ops->num_queues > VIRTIO_MAX_VQUEUES) {
        log_error("invalid queue count %u for %s", ops->num_queues,
                  virtio_device_type_to_string(ops->type));
        return -EINVAL;
    }

    vdev->vqs_len = ops->num_queues;
    VirtQueue *vqs = calloc(ops->num_queues, sizeof(VirtQueue));
    if (!vqs)
        return -ENOMEM;

    for (uint32_t i = 0; i < ops->num_queues; i++) {
        virtqueue_reset(&vqs[i], i);
        vqs[i].queue_num_max = ops->queue_max_size;
        vqs[i].dev = vdev;
        if (ops->notify_handlers[i])
            vqs[i].notify_handler = ops->notify_handlers[i];
    }
    vdev->vqs = vqs;
    return 0;
}

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type) {
    log_info("initializing mmio registers for %s",
             virtio_device_type_to_string(type));
    regs->device_id = type;
    regs->queue_sel = 0;
}

void virtio_dev_reset(VirtIODevice *vdev) {
    // When driver read first 4 encoded messages, it will reset dev.
    log_debug("virtio dev reset");
    pthread_mutex_lock(&vdev->interrupt_lock);
    if (vdev->interrupt_line_asserted) {
        if (virtio_deassert_line_locked(vdev) == 0) {
            vdev->interrupt_line_asserted = false;
        }
    }
    if (!vdev->interrupt_line_asserted) {
        vdev->regs.interrupt_status = 0;
    }
    pthread_mutex_unlock(&vdev->interrupt_lock);
    vdev->regs.status = 0;
    int idx = vdev->regs.queue_sel;
    vdev->vqs[idx].ready = 0;
    // Run the device reset op before re-initializing the virtqueues: reset
    // ops (e.g. virtio-blk's) quiesce worker threads that touch the vq
    // structs, which must not race with virtqueue_reset() below.
    const struct virtio_device_ops *ops = lookup_ops(vdev->type);
    if (ops && ops->reset)
        ops->reset(vdev);
    for (uint32_t i = 0; i < vdev->vqs_len; i++) {
        virtqueue_reset(&vdev->vqs[i], i);
    }
    vdev->activated = false;
}

void virtqueue_reset(VirtQueue *vq, int idx) {
    // Reserve these fields
    void *addr = vq->notify_handler;
    VirtIODevice *dev = vq->dev;
    uint32_t queue_num_max = vq->queue_num_max;

    // Clear others
    memset(vq, 0, sizeof(VirtQueue));
    vq->vq_idx = idx;
    vq->notify_handler = addr;
    vq->dev = dev;
    vq->queue_num_max = queue_num_max;
    pthread_mutex_init(&vq->used_ring_lock, NULL);
}

// check if virtqueue has new requests
bool virtqueue_is_empty(VirtQueue *vq) { return vq_is_empty(vq); }

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx) {
    if (desc_table[idx].flags & VRING_DESC_F_WRITE)
        return true;
    return false;
}

void *get_virt_addr(void *zonex_ipa, int zone_id) {
    if (zone_id < 0 || zone_id >= MAX_ZONES)
        return NULL;

    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    struct zone_mem *z = &zone_mem[zone_id];
    uintptr_t ipa = (uintptr_t)zonex_ipa;

    for (size_t i = 0; i < z->num_regions; i++) {
        uintptr_t lef = z->regions[i].zonex_ipa;
        uintptr_t rig = z->regions[i].zonex_ipa + z->regions[i].mem_size;
        if (lef <= ipa && ipa < rig) {
            void *virt_addr = (void *)(ipa - lef + z->regions[i].virt_addr);
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            return virt_addr;
        }
    }

    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    log_error("can't find zone mem index for zonex_ipa = 0x%" PRIxPTR, ipa);
    return NULL;
}

// When virtio device is processing virtqueue, driver adding an elem to
// virtqueue is no need to notify device.
void virtqueue_disable_notify(VirtQueue *vq) {
    if (vq->event_idx_enabled) {
        VQ_AVAIL_EVENT(vq) = vq->last_avail_idx - 1;
    } else {
        vq->used_ring->flags |= (uint16_t)VRING_USED_F_NO_NOTIFY;
    }
    write_barrier();
}

void virtqueue_enable_notify(VirtQueue *vq) {
    if (vq->event_idx_enabled) {
        VQ_AVAIL_EVENT(vq) = vq->avail_ring->idx;
    } else {
        vq->used_ring->flags &= ~(uint16_t)VRING_USED_F_NO_NOTIFY;
    }
    write_barrier();
}

void virtqueue_set_desc_table(VirtQueue *vq) {
    int zone_id = vq->dev->zone_id;
    log_debug("zone %d set dev %s desc table ipa at %#x", zone_id,
              virtio_device_type_to_string(vq->dev->type), vq->desc_table_addr);
    vq->desc_table = (VirtqDesc *)get_virt_addr(
        (void *)(uintptr_t)vq->desc_table_addr, zone_id);
}

void virtqueue_set_avail(VirtQueue *vq) {
    int zone_id = vq->dev->zone_id;
    log_debug("zone %d set dev %s avail ring ipa at %#x", zone_id,
              virtio_device_type_to_string(vq->dev->type), vq->avail_addr);
    vq->avail_ring =
        (VirtqAvail *)get_virt_addr((void *)(uintptr_t)vq->avail_addr, zone_id);
}

void virtqueue_set_used(VirtQueue *vq) {
    int zone_id = vq->dev->zone_id;
    log_debug("zone %d set dev %s used ring ipa at %#x", zone_id,
              virtio_device_type_to_string(vq->dev->type), vq->used_addr);
    vq->used_ring =
        (VirtqUsed *)get_virt_addr((void *)(uintptr_t)vq->used_addr, zone_id);
}

// record one descriptor to iov.
inline int descriptor2iov(int i, volatile VirtqDesc *vd, struct iovec *iov,
                          uint16_t *flags, int zone_id, bool copy_flags) {
    void *host_addr;

    host_addr = get_virt_addr((void *)vd->addr, zone_id);
    iov[i].iov_base = host_addr;
    iov[i].iov_len = vd->len;
    // log_debug("vd->addr ipa is %x, iov_base is %x, iov_len is %d", vd->addr,
    // host_addr, vd->len);
    if (copy_flags)
        flags[i] = vd->flags;

    return 0;
}

// Dispatch a descriptor into cfg->out_iov or cfg->in_iov based on flags.
static inline int push_descriptor(uint16_t flags, void *address,
                                  uint32_t length,
                                  const struct VirtioBufConfig *cfg,
                                  size_t *out_count, size_t *in_count) {
    struct iovec *buffer;
    size_t max;
    size_t *count;

    if (flags & VRING_DESC_F_WRITE) {
        buffer = cfg->in_iov;
        max = cfg->max_in;
        count = in_count;
    } else {
        buffer = cfg->out_iov;
        max = cfg->max_out;
        count = out_count;
    }

    if (*count >= max)
        return -1;

    buffer[*count].iov_base = address;
    buffer[*count].iov_len = length;
    (*count)++;
    return 0;
}

/// record one descriptor list to iov, caller-provided buffer (no malloc)
int process_descriptor_chain_buf(VirtQueue *virtqueue, uint16_t descriptor_head,
                                 const struct VirtioBufConfig *cfg,
                                 struct VirtioRequest *req) {

    // Single-pass traversal; virtqueue->num guards against circular chains.
    size_t out_count = 0, in_count = 0;
    uint16_t next = descriptor_head;
    volatile VirtqDesc *descriptor_table = virtqueue->desc_table;
    for (size_t iter = 0; iter < virtqueue->num; iter++) {
        VirtqDesc descriptor = descriptor_table[next];

        if (descriptor.flags & VRING_DESC_F_INDIRECT) {
            const size_t indirect_count = descriptor.len / sizeof(VirtqDesc);

            volatile VirtqDesc *indirect_table = get_virt_addr(
                (void *)(uintptr_t)descriptor.addr, virtqueue->dev->zone_id);
            uint16_t indirect_next = 0;
            for (size_t j = 0; j < indirect_count; j++) {
                if (indirect_next >= indirect_count) {
                    log_error("indirect_next is not less than indirect_count: "
                              "%zu >= %zu",
                              indirect_next, indirect_count);
                    return -1;
                }

                VirtqDesc indirect_descriptor = indirect_table[indirect_next];
                void *address =
                    get_virt_addr((void *)(uintptr_t)indirect_descriptor.addr,
                                  virtqueue->dev->zone_id);
                if (!address) {
                    log_error("address is null");
                    return -1;
                }

                if (push_descriptor(indirect_descriptor.flags, address,
                                    indirect_descriptor.len, cfg, &out_count,
                                    &in_count) < 0) {
                    log_error("descriptor buffer overflow");
                    return -1;
                }
                if (!(indirect_descriptor.flags & VRING_DESC_F_NEXT))
                    break;
                indirect_next = indirect_descriptor.next;
            }
        } else {
            void *address = get_virt_addr((void *)(uintptr_t)descriptor.addr,
                                          virtqueue->dev->zone_id);
            if (!address) {
                log_error("address is null");
                return -1;
            }

            if (push_descriptor(descriptor.flags, address, descriptor.len, cfg,
                                &out_count, &in_count) < 0) {
                log_error("descriptor buffer overflow");
                return -1;
            }
        }

        if (!(descriptor.flags & VRING_DESC_F_NEXT))
            break;
        next = descriptor.next;
    }

    *req = (struct VirtioRequest){
        .out_iov = cfg->out_iov,
        .out_count = out_count,
        .in_iov = cfg->in_iov,
        .in_count = in_count,
    };

    virtqueue->last_avail_idx++;
    return out_count + in_count;
}

/// record one descriptor list to iov
/// \param desc_idx the first descriptor's idx in descriptor list.
/// \param iov the iov to record
/// \param flags each descriptor's flags
/// \param append_len the number of iovs to append
/// \return the len of iovs
int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                             struct iovec **iov, uint16_t **flags,
                             int append_len, bool copy_flags) {
    uint16_t next, last_avail_idx;
    volatile VirtqDesc *vdesc, *ind_table, *ind_desc;
    int chain_len = 0, i, table_len;

    // idx is the last available index processed during the last kick
    last_avail_idx = vq->last_avail_idx;

    // No new requests
    if (last_avail_idx == vq->avail_ring->idx)
        return 0;

    // Update to the index to be processed during this kick
    vq->last_avail_idx++;

    // Get the index of the first available descriptor
    *desc_idx = next = vq->avail_ring->ring[last_avail_idx & (vq->num - 1)];
    // Record the length of the descriptor chain to chain_len
    for (i = 0; i < (int)vq->num; i++, next = vdesc->next) {
        // Get a descriptor
        vdesc = &vq->desc_table[next];
        // TODO: vdesc->len may not be chain_len, virtio specification doesn't
        // say it.

        // Check if this descriptor supports the VRING_DESC_F_INDIRECT feature
        // If supported, it means that the descriptor points to a set of
        // descriptors, i.e., one descriptor can describe multiple scattered
        // buffers
        if (vdesc->flags & VRING_DESC_F_INDIRECT) {
            chain_len +=
                vdesc->len / 16; // This descriptor points to 16 descriptors
            i--;
        }
        // Exit if there is no next descriptor
        if ((vdesc->flags & VRING_DESC_F_NEXT) == 0)
            break;
    }

    // Update chain length and reset next to the first descriptor
    chain_len += i + 1, next = *desc_idx;

    // Allocate a buffer for each descriptor, using iov to manage them uniformly
    *iov = malloc(sizeof(struct iovec) * (chain_len + append_len));
    if (copy_flags)
        // Record the flag of each descriptor
        *flags = malloc(sizeof(uint16_t) * (chain_len + append_len));

    // Traverse the descriptor chain and copy the buffer pointed to by each
    // descriptor to iov
    for (i = 0; i < chain_len; i++, next = vdesc->next) {
        vdesc = &vq->desc_table[next];
        // If the descriptor supports the VRING_DESC_F_INDIRECT feature
        if (vdesc->flags & VRING_DESC_F_INDIRECT) {
            // Get the address of the indirect table pointed to by this
            // descriptor
            ind_table = (VirtqDesc *)(get_virt_addr((void *)vdesc->addr,
                                                    vq->dev->zone_id));
            table_len = vdesc->len / 16;
            log_debug("find indirect desc, table_len is %d", table_len);
            next = 0;
            for (;;) {
                // log_debug("indirect desc next is %d", next);
                ind_desc = &ind_table[next];
                descriptor2iov(i, ind_desc, *iov, flags == NULL ? NULL : *flags,
                               vq->dev->zone_id, copy_flags);
                table_len--;
                i++;
                // No more next descriptor
                if ((ind_desc->flags & VRING_DESC_F_NEXT) == 0)
                    break;
                next = ind_desc->next;
            }
            if (table_len != 0) {
                log_error("invalid indirect descriptor chain");
                break;
            }
        } else {
            // For a normal descriptor, copy it directly to iov
            descriptor2iov(i, vdesc, *iov, flags == NULL ? NULL : *flags,
                           vq->dev->zone_id, copy_flags);
        }
    }
    return chain_len;
}

void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen) {
    volatile VirtqUsed *used_ring;
    volatile VirtqUsedElem *elem;
    uint16_t used_idx, mask;
    // There is no need to worry about if used_ring is full, because used_ring's
    // len is equal to descriptor table's.
    write_barrier();
    // pthread_mutex_lock(&vq->used_ring_lock);
    used_ring = vq->used_ring;
    used_idx = used_ring->idx;
    mask = vq->num - 1;
    elem = &used_ring->ring[used_idx++ & mask];
    elem->id = idx;
    elem->len = iolen;
    used_ring->idx = used_idx;
    write_barrier();
    // pthread_mutex_unlock(&vq->used_ring_lock);
    log_debug(
        "update used ring: used_idx is %d, elem->idx is %d, vq->num is %d",
        used_idx, idx, vq->num);
}

void update_used_ring_batch(VirtQueue *vq, const uint16_t *indices,
                            const uint32_t *lens, size_t count) {
    volatile VirtqUsed *used_ring;
    uint16_t used_idx, mask;

    if (count == 0)
        return;

    // Ensure prior stores (e.g. readv into guest buffers) are globally
    // visible before any used-ring entry becomes observable.
    write_barrier();

    used_ring = vq->used_ring;
    used_idx = used_ring->idx;
    mask = vq->num - 1;

    for (size_t i = 0; i < count; i++) {
        used_ring->ring[(used_idx + i) & mask].id = indices[i];
        used_ring->ring[(used_idx + i) & mask].len = lens[i];
    }

    write_barrier(); // make all entries visible
    used_ring->idx = used_idx + count;
    write_barrier(); // make idx update visible
}

// function for translating virtio offset to meaning string
static const char *virtio_mmio_reg_name(uint64_t offset) {
    switch (offset) {
    case VIRTIO_MMIO_MAGIC_VALUE:
        return "VIRTIO_MMIO_MAGIC_VALUE";
    case VIRTIO_MMIO_VERSION:
        return "VIRTIO_MMIO_VERSION";
    case VIRTIO_MMIO_DEVICE_ID:
        return "VIRTIO_MMIO_DEVICE_ID";
    case VIRTIO_MMIO_VENDOR_ID:
        return "VIRTIO_MMIO_VENDOR_ID";
    case VIRTIO_MMIO_DEVICE_FEATURES:
        return "VIRTIO_MMIO_DEVICE_FEATURES";
    case VIRTIO_MMIO_DEVICE_FEATURES_SEL:
        return "VIRTIO_MMIO_DEVICE_FEATURES_SEL";
    case VIRTIO_MMIO_DRIVER_FEATURES:
        return "VIRTIO_MMIO_DRIVER_FEATURES";
    case VIRTIO_MMIO_DRIVER_FEATURES_SEL:
        return "VIRTIO_MMIO_DRIVER_FEATURES_SEL";
    case VIRTIO_MMIO_GUEST_PAGE_SIZE:
        return "VIRTIO_MMIO_GUEST_PAGE_SIZE";
    case VIRTIO_MMIO_QUEUE_SEL:
        return "VIRTIO_MMIO_QUEUE_SEL";
    case VIRTIO_MMIO_QUEUE_NUM_MAX:
        return "VIRTIO_MMIO_QUEUE_NUM_MAX";
    case VIRTIO_MMIO_QUEUE_NUM:
        return "VIRTIO_MMIO_QUEUE_NUM";
    case VIRTIO_MMIO_QUEUE_ALIGN:
        return "VIRTIO_MMIO_QUEUE_ALIGN";
    case VIRTIO_MMIO_QUEUE_PFN:
        return "VIRTIO_MMIO_QUEUE_PFN";
    case VIRTIO_MMIO_QUEUE_READY:
        return "VIRTIO_MMIO_QUEUE_READY";
    case VIRTIO_MMIO_QUEUE_NOTIFY:
        return "VIRTIO_MMIO_QUEUE_NOTIFY";
    case VIRTIO_MMIO_INTERRUPT_STATUS:
        return "VIRTIO_MMIO_INTERRUPT_STATUS";
    case VIRTIO_MMIO_INTERRUPT_ACK:
        return "VIRTIO_MMIO_INTERRUPT_ACK";
    case VIRTIO_MMIO_STATUS:
        return "VIRTIO_MMIO_STATUS";
    case VIRTIO_MMIO_QUEUE_DESC_LOW:
        return "VIRTIO_MMIO_QUEUE_DESC_LOW";
    case VIRTIO_MMIO_QUEUE_DESC_HIGH:
        return "VIRTIO_MMIO_QUEUE_DESC_HIGH";
    case VIRTIO_MMIO_QUEUE_AVAIL_LOW:
        return "VIRTIO_MMIO_QUEUE_AVAIL_LOW";
    case VIRTIO_MMIO_QUEUE_AVAIL_HIGH:
        return "VIRTIO_MMIO_QUEUE_AVAIL_HIGH";
    case VIRTIO_MMIO_QUEUE_USED_LOW:
        return "VIRTIO_MMIO_QUEUE_USED_LOW";
    case VIRTIO_MMIO_QUEUE_USED_HIGH:
        return "VIRTIO_MMIO_QUEUE_USED_HIGH";
    case VIRTIO_MMIO_CONFIG_GENERATION:
        return "VIRTIO_MMIO_CONFIG_GENERATION";
    case VIRTIO_MMIO_CONFIG:
        return "VIRTIO_MMIO_CONFIG";
    default:
        return "UNKNOWN";
    }
}

uint64_t virtio_mmio_read(VirtIODevice *vdev, uint64_t offset, unsigned size) {
    log_debug("READ virtio mmio at offset=%#x[%s], size=%d, vdev=%p, type=%d",
              offset, virtio_mmio_reg_name(offset), size, vdev, vdev->type);

    if (!vdev) {
        switch (offset) {
        case VIRTIO_MMIO_MAGIC_VALUE:
            log_debug("read VIRTIO_MMIO_MAGIC_VALUE");
            return VIRT_MAGIC;
        case VIRTIO_MMIO_VERSION:
            log_debug("read VIRTIO_MMIO_VERSION");
            return VIRT_VERSION;
        case VIRTIO_MMIO_VENDOR_ID:
            log_debug("read VIRTIO_MMIO_VENDOR_ID");
            return VIRT_VENDOR;
        default:
            return 0;
        }
    }

    if (offset >= VIRTIO_MMIO_CONFIG) {
        offset -= VIRTIO_MMIO_CONFIG;
        // the first member of vdev->dev must be config.
        log_debug("read virtio dev config");
        return *(uint64_t *)((uintptr_t)vdev->dev + offset);
    }

    if (size != 4) {
        log_error("virtio-mmio-read: wrong size access to register!");
        return 0;
    }

    switch (offset) {
    case VIRTIO_MMIO_MAGIC_VALUE:
        log_debug("read VIRTIO_MMIO_MAGIC_VALUE");
        return VIRT_MAGIC;
    case VIRTIO_MMIO_VERSION:
        log_debug("read VIRTIO_MMIO_VERSION");
        return VIRT_VERSION;
    case VIRTIO_MMIO_DEVICE_ID:
        log_debug("read VIRTIO_MMIO_DEVICE_ID");
        return vdev->regs.device_id;
    case VIRTIO_MMIO_VENDOR_ID:
        log_debug("read VIRTIO_MMIO_VENDOR_ID");
        return VIRT_VENDOR;
    case VIRTIO_MMIO_DEVICE_FEATURES:
        log_debug("read VIRTIO_MMIO_DEVICE_FEATURES");

        if (vdev->regs.dev_feature_sel) {
            return vdev->regs.dev_feature >> 32;
        } else {
            return vdev->regs.dev_feature;
        }
    case VIRTIO_MMIO_QUEUE_NUM_MAX:
        log_debug("read VIRTIO_MMIO_QUEUE_NUM_MAX");
        if (vdev->regs.queue_sel >= vdev->vqs_len) {
            return 0;
        }
        return vdev->vqs[vdev->regs.queue_sel].queue_num_max;
    case VIRTIO_MMIO_QUEUE_READY:
        log_debug("read VIRTIO_MMIO_QUEUE_READY");
        return vdev->vqs[vdev->regs.queue_sel].ready;
    case VIRTIO_MMIO_INTERRUPT_STATUS: {
        pthread_mutex_lock(&vdev->interrupt_lock);
        uint32_t interrupt_status = vdev->regs.interrupt_status;
        pthread_mutex_unlock(&vdev->interrupt_lock);
        uint64_t trace_seq = atomic_fetch_add_explicit(&virtio_irq_trace_seq, 1,
                                                       memory_order_relaxed);
        if (virtio_trace_sample(trace_seq)) {
            log_info("[VDBG:status-read] seq=%llu zone=%u dev=%s irq=%u "
                     "status=%#x",
                     (unsigned long long)trace_seq, vdev->zone_id,
                     virtio_device_type_to_string(vdev->type), vdev->irq_id,
                     interrupt_status);
        }
        return interrupt_status;
    }
    case VIRTIO_MMIO_STATUS:
        log_debug("read VIRTIO_MMIO_STATUS");
        return vdev->regs.status;
    case VIRTIO_MMIO_CONFIG_GENERATION:
        log_debug("read VIRTIO_MMIO_CONFIG_GENERATION");
        return vdev->regs.generation;
    case VIRTIO_MMIO_DEVICE_FEATURES_SEL:
    case VIRTIO_MMIO_DRIVER_FEATURES:
    case VIRTIO_MMIO_DRIVER_FEATURES_SEL:
    case VIRTIO_MMIO_QUEUE_SEL:
    case VIRTIO_MMIO_QUEUE_NUM:
    case VIRTIO_MMIO_QUEUE_NOTIFY:
    case VIRTIO_MMIO_INTERRUPT_ACK:
    case VIRTIO_MMIO_QUEUE_DESC_LOW:
    case VIRTIO_MMIO_QUEUE_DESC_HIGH:
    case VIRTIO_MMIO_QUEUE_AVAIL_LOW:
    case VIRTIO_MMIO_QUEUE_AVAIL_HIGH:
    case VIRTIO_MMIO_QUEUE_USED_LOW:
    case VIRTIO_MMIO_QUEUE_USED_HIGH:
        log_error("read of write-only register");
        return 0;
    default:
        log_error("bad register offset %#x", offset);
        return 0;
    }
    return 0;
}

void virtio_mmio_write(VirtIODevice *vdev, uint64_t offset, uint64_t value,
                       unsigned size) {
    log_debug("WRITE virtio mmio at offset=%#x[%s], value=%#x, size=%d, "
              "vdev=%p, type=%d",
              offset, virtio_mmio_reg_name(offset), value, size, vdev,
              vdev->type);

    VirtMmioRegs *regs = &vdev->regs;
    VirtQueue *vqs = vdev->vqs;
    if (!vdev) {
        return;
    }

    if (offset >= VIRTIO_MMIO_CONFIG) {
        offset -= VIRTIO_MMIO_CONFIG;
        log_error("virtio_mmio_write: can't write config space");
        return;
    }
    if (size != 4) {
        log_error("virtio_mmio_write: wrong size access to register!");
        return;
    }

    switch (offset) {
    case VIRTIO_MMIO_DEVICE_FEATURES_SEL:
        log_debug("write VIRTIO_MMIO_DEVICE_FEATURES_SEL");
        if (value) {
            regs->dev_feature_sel = 1;
        } else {
            regs->dev_feature_sel = 0;
        }
        break;
    case VIRTIO_MMIO_DRIVER_FEATURES:
        log_debug("zone %d driver set device %s, accepted features %d",
                  vdev->zone_id, virtio_device_type_to_string(vdev->type),
                  value);
        if (regs->drv_feature_sel) {
            regs->drv_feature |= value << 32;
        } else {
            regs->drv_feature |= value;
        }

        // If the driver frontend has activated VIRTIO_RING_F_EVENT_IDX, enable
        // the related settings
        if (regs->drv_feature & (1ULL << VIRTIO_RING_F_EVENT_IDX)) {
            log_debug("zone %d driver accepted VIRTIO_RING_F_EVENT_IDX",
                      vdev->zone_id);
            int len = vdev->vqs_len;
            for (int i = 0; i < len; i++)
                vqs[i].event_idx_enabled = 1;
        }
        break;
    case VIRTIO_MMIO_DRIVER_FEATURES_SEL:
        log_debug("write VIRTIO_MMIO_DRIVER_FEATURES_SEL");

        if (value) {
            regs->drv_feature_sel = 1;
        } else {
            regs->drv_feature_sel = 0;
        }
        break;
    case VIRTIO_MMIO_QUEUE_SEL:
        log_debug("zone %d driver set device %s, selecting queue %d",
                  vdev->zone_id, virtio_device_type_to_string(vdev->type),
                  value);
        regs->queue_sel = value;
        break;
    case VIRTIO_MMIO_QUEUE_NUM:
        log_debug("zone %d driver set device %s, use virtqueue num %d",
                  vdev->zone_id, virtio_device_type_to_string(vdev->type),
                  value);

        vqs[regs->queue_sel].num = value;
        break;
    case VIRTIO_MMIO_QUEUE_READY:
        log_debug("write VIRTIO_MMIO_QUEUE_READY");

        vqs[regs->queue_sel].ready = value;
        break;
    case VIRTIO_MMIO_QUEUE_NOTIFY:
        log_debug("****** zone %d %s queue notify begin ******", vdev->zone_id,
                  virtio_device_type_to_string(vdev->type));

        if (value < vdev->vqs_len && vqs[value].notify_handler) {
            log_debug("queue notify ready, handler addr is %#x",
                      vqs[value].notify_handler);
            vqs[value].notify_handler(vdev, &vqs[value]);
        } else {
            log_warn("zone %d %s: ignoring queue notify, value %" PRIu64
                     ", vqs_len %u",
                     vdev->zone_id, virtio_device_type_to_string(vdev->type),
                     value, vdev->vqs_len);
        }

        log_debug("****** zone %d %s queue notify end ******", vdev->zone_id,
                  virtio_device_type_to_string(vdev->type));

        break;
    case VIRTIO_MMIO_INTERRUPT_ACK: {
        pthread_mutex_lock(&vdev->interrupt_lock);
        uint32_t status_before = regs->interrupt_status;
        uint32_t ack =
            (uint32_t)value & (VIRTIO_MMIO_INT_VRING | VIRTIO_MMIO_INT_CONFIG);
        uint32_t status_after = status_before & ~ack;
        int deassert_ret = 0;
        if (status_before != 0 && status_after == 0 &&
            vdev->interrupt_line_asserted) {
            deassert_ret = virtio_deassert_line_locked(vdev);
        }
        if (deassert_ret == 0) {
            regs->interrupt_status = status_after;
            if (status_after == 0)
                vdev->interrupt_line_asserted = false;
        } else {
            status_after = status_before;
        }
        uint64_t trace_seq = atomic_fetch_add_explicit(&virtio_irq_trace_seq, 1,
                                                       memory_order_relaxed);
        if (virtio_trace_sample(trace_seq)) {
            log_info("[VDBG:ack] seq=%llu zone=%u dev=%s irq=%u before=%#x "
                     "ack=%#x after=%#x deassert_ret=%d",
                     (unsigned long long)trace_seq, vdev->zone_id,
                     virtio_device_type_to_string(vdev->type), vdev->irq_id,
                     status_before, ack, status_after, deassert_ret);
        }
        pthread_mutex_unlock(&vdev->interrupt_lock);
        break;
    }
    case VIRTIO_MMIO_STATUS:
        log_debug("write VIRTIO_MMIO_STATUS");

        regs->status = value;
        if (regs->status == 0) {
            virtio_dev_reset(vdev);
        }
        if (vdev->status_changed)
            vdev->status_changed(vdev, value);
        break;
    case VIRTIO_MMIO_QUEUE_DESC_LOW:
        log_debug("write VIRTIO_MMIO_QUEUE_DESC_LOW");

        vqs[regs->queue_sel].desc_table_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_DESC_HIGH:
        log_debug("write VIRTIO_MMIO_QUEUE_DESC_HIGH");

        vqs[regs->queue_sel].desc_table_addr |= value << 32;
        virtqueue_set_desc_table(&vqs[regs->queue_sel]);
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_LOW:
        log_debug("write VIRTIO_MMIO_QUEUE_AVAIL_LOW");

        vqs[regs->queue_sel].avail_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_HIGH:
        log_debug("write VIRTIO_MMIO_QUEUE_AVAIL_HIGH");

        vqs[regs->queue_sel].avail_addr |= value << 32;
        virtqueue_set_avail(&vqs[regs->queue_sel]);
        break;
    case VIRTIO_MMIO_QUEUE_USED_LOW:
        log_debug("write VIRTIO_MMIO_QUEUE_USED_LOW");

        vqs[regs->queue_sel].used_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_USED_HIGH:
        log_debug("write VIRTIO_MMIO_QUEUE_USED_HIGH");

        vqs[regs->queue_sel].used_addr |= value << 32;
        virtqueue_set_used(&vqs[regs->queue_sel]);
        break;
    case VIRTIO_MMIO_MAGIC_VALUE:
    case VIRTIO_MMIO_VERSION:
    case VIRTIO_MMIO_DEVICE_ID:
    case VIRTIO_MMIO_VENDOR_ID:
    case VIRTIO_MMIO_DEVICE_FEATURES:
    case VIRTIO_MMIO_QUEUE_NUM_MAX:
    case VIRTIO_MMIO_INTERRUPT_STATUS:
    case VIRTIO_MMIO_CONFIG_GENERATION:
        log_error("%s: write to read-only register 0#x", __func__, offset);
        break;

    default:
        log_error("%s: bad register offset 0#x", __func__, offset);
    }
}

inline bool in_range(uint64_t value, uint64_t lower, uint64_t len) {
    return ((value >= lower) && (value < (lower + len)));
}

// Inject irq_id to target zone. It will add to res list, and notify hypervisor
// through ioctl.
void virtio_inject_irq(VirtQueue *vq) {
    uint16_t last_used_idx, idx, event_idx;
    last_used_idx = vq->last_used_idx;
    vq->last_used_idx = idx = vq->used_ring->idx;
    // read_barrier();
    if (idx == last_used_idx) {
        log_debug("idx equals last_used_idx");
        return;
    }
    if (!vq->event_idx_enabled &&
        (vq->avail_ring->flags & VRING_AVAIL_F_NO_INTERRUPT)) {
        log_debug("no interrupt");
        return;
    }
    if (vq->event_idx_enabled) {
        event_idx = VQ_USED_EVENT(vq);
        log_debug("idx is %d, event_idx is %d, last_used_idx is %d", idx,
                  event_idx, last_used_idx);
        if (!vring_need_event(event_idx, idx, last_used_idx)) {
            return;
        }
    }
    pthread_mutex_lock(&vq->dev->interrupt_lock);
    vq->dev->regs.interrupt_status |= VIRTIO_MMIO_INT_VRING;
    if (vq->dev->interrupt_line_asserted) {
        uint64_t trace_seq = atomic_fetch_add_explicit(&virtio_irq_trace_seq, 1,
                                                       memory_order_relaxed);
        if (virtio_trace_sample(trace_seq)) {
            log_info("[VDBG:assert-merge] seq=%llu zone=%u dev=%s irq=%u "
                     "status=%#x",
                     (unsigned long long)trace_seq, vq->dev->zone_id,
                     virtio_device_type_to_string(vq->dev->type),
                     vq->dev->irq_id, vq->dev->regs.interrupt_status);
        }
        pthread_mutex_unlock(&vq->dev->interrupt_lock);
        return;
    }
    volatile struct device_res *res;

    // virtio_bridge is a global resource located in shared memory.
    // Access to critical resources such as res_front and res_rear requires
    // locking.

    // Since the shared resources related to res_list are only accessed
    //  at one specific code location, a lock before polling is_queue_full
    //  is enough to ensure thread safety and performance.
    pthread_mutex_lock(&RES_MUTEX);

    while (is_queue_full(virtio_bridge->res_front, virtio_bridge->res_rear,
                         MAX_REQ)) {
    }
    unsigned int res_rear = virtio_bridge->res_rear;
    res = &virtio_bridge->res_list[res_rear];
    res->irq_id = vq->dev->irq_id;
    res->target_zone = vq->dev->zone_id;
    write_barrier();
    virtio_bridge->res_rear = (res_rear + 1) & (MAX_REQ - 1);
    write_barrier();
    pthread_mutex_unlock(&RES_MUTEX);
    int ret;
    do {
        ret = ioctl(ko_fd, HVISOR_FINISH_REQ);
    } while (ret < 0 && errno == EINTR);
    if (ret < 0) {
        log_error("assert failed: zone=%u irq=%u errno=%d (%s)",
                  vq->dev->zone_id, vq->dev->irq_id, errno, strerror(errno));
    }
    if (ret == 0)
        vq->dev->interrupt_line_asserted = true;
    uint64_t trace_seq = atomic_fetch_add_explicit(&virtio_irq_trace_seq, 1,
                                                   memory_order_relaxed);
    if (virtio_trace_sample(trace_seq)) {
        log_info("[VDBG:assert] seq=%llu zone=%u dev=%s irq=%u vq=%u "
                 "status=%#x line=%u ret=%d",
                 (unsigned long long)trace_seq, vq->dev->zone_id,
                 virtio_device_type_to_string(vq->dev->type), vq->dev->irq_id,
                 (unsigned)vq->vq_idx, vq->dev->regs.interrupt_status,
                 vq->dev->interrupt_line_asserted, ret);
    }
    pthread_mutex_unlock(&vq->dev->interrupt_lock);
}

void virtio_finish_cfg_req(uint32_t target_cpu, uint64_t value) {
    virtio_bridge->cfg_values[target_cpu] = value;
    write_barrier();
    virtio_bridge->cfg_flags[target_cpu]++;
    write_barrier();
}

int virtio_handle_req(volatile struct device_req *req) {
    uint64_t value = 0;
    VirtIODevice *vdev = NULL;

    // Check if the request corresponds to a virtio device in a specific zone
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; ++i) {
        if ((req->src_zone == vdevs[i]->zone_id) &&
            in_range(req->address, vdevs[i]->base_addr, vdevs[i]->len)) {
            vdev = vdevs[i];
            break;
        }
    }
    pthread_mutex_unlock(&VDEV_MUTEX);

    if (!vdev) {
        log_warn("no matched virtio dev in zone %d, address is 0x%x",
                 req->src_zone, req->address);
        // No device at this address; return 0 so the guest sees an absent
        // device (MagicValue != VIRT_MAGIC).
        virtio_finish_cfg_req(req->src_cpu, 0);
        return -1;
    }

    uint64_t offs = req->address - vdev->base_addr;

    // Write or read the device's MMIO register
    if (req->is_write) {
        virtio_mmio_write(vdev, offs, req->value, req->size);
    } else {
        value = virtio_mmio_read(vdev, offs, req->size);
        log_debug("read value is 0x%x", value);
    }

    // Control instructions do not require interrupts to return data
    // The requester will block and wait
    if (!req->need_interrupt) {
        // If a request is a control not a data request
        virtio_finish_cfg_req(req->src_cpu, value);
    }

    log_debug("src_zone is %d, src_cpu is %lld", req->src_zone, req->src_cpu);
    return 0;
}

void virtio_close() {
    log_warn("virtio devices will be closed");
    stop_virtio_control_server();
    destroy_event_monitor();
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; i++) {
        if (vdevs[i] && vdevs[i]->virtio_close)
            vdevs[i]->virtio_close(vdevs[i]);
        vdevs[i] = NULL;
    }
    vdevs_num = 0;
    pthread_mutex_unlock(&VDEV_MUTEX);
    if (ko_fd >= 0) {
        close(ko_fd);
        ko_fd = -1;
    }

    if (efd >= 0) {
        close(efd);
        efd = -1;
    }

    if (sfd >= 0) {
        close(sfd);
        sfd = -1;
    }

    if (epoll_fd >= 0) {
        close(epoll_fd);
        epoll_fd = -1;
    }

    if (virtio_bridge && virtio_bridge != MAP_FAILED) {
        munmap((void *)virtio_bridge, MMAP_SIZE);
        virtio_bridge = NULL;
    }
    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    for (int i = 0; i < MAX_ZONES; i++) {
        struct zone_mem *z = &zone_mem[i];
        for (size_t j = 0; j < z->num_regions; j++)
            if (z->regions[j].mem_size != 0)
                munmap((void *)z->regions[j].virt_addr, z->regions[j].mem_size);
        memset(z, 0, sizeof(*z));
    }
    pthread_mutex_unlock(&ZONE_MEM_MUTEX);

    log_warn("virtio daemon exit successfully");
}

// Ensure MAX_REQ is a power of two for bitwise masking to work correctly.
_Static_assert((MAX_REQ != 0) && ((MAX_REQ & (MAX_REQ - 1)) == 0),
               "MAX_REQ must be a power of 2");

/**
 * @brief Consumes pending VirtIO requests from the shared ring buffer
 *
 * This function implements a high-performance consumer that processes VirtIO
 * requests from a circular shared buffer. It uses atomic operations and
 * Dekker's algorithm to avoid race conditions and minimize unnecessary wakeups.
 *
 * The function performs the following operations:
 * - Reads requests from the shared ring buffer using atomic operations
 * - Processes each request by calling virtio_handle_req()
 * - Implements busy-polling with a defined maximum count to avoid excessive CPU
 * usage
 * - Uses Dekker's algorithm to coordinate with the producer for efficient
 * sleep/wakeup synchronization
 *
 * @return the number of requests successfully processed during this invocation
 */
static int consume_pending_requests(void) {
    int proc_count = 0;

    const uint32_t MAX_POLL_COUNT = 10000000;
    uint32_t poll_count = 0;

    // Pointers to indices in the shared virtio_bridge structure.
    uint32_t *p_req_front = (uint32_t *)&virtio_bridge->req_front;
    uint32_t *p_req_rear = (uint32_t *)&virtio_bridge->req_rear;
    uint8_t *p_need_wakeup = (uint8_t *)&virtio_bridge->need_wakeup;

    // Local copy of the front index to minimize shared memory reads
    uint32_t req_front = __atomic_load_n(p_req_front, memory_order_relaxed);

    // Inform the producer that we are active; no need to trigger eventfd
    __atomic_store_n(p_need_wakeup, 0, memory_order_relaxed);

    while (true) {
        uint32_t req_rear = __atomic_load_n(p_req_rear, memory_order_relaxed);
        if (req_front != req_rear) {
            // Make Guest data visible to Host
            __atomic_thread_fence(memory_order_acquire);

            // Data available: reset poll counter and clear sleep intention
            poll_count = 0;
            ++proc_count;

            struct device_req *req =
                (struct device_req *)&virtio_bridge->req_list[req_front];
            virtio_handle_req(req);

            // Move to the next slot in the circular buffer
            req_front = (req_front + 1U) & (uint32_t)(MAX_REQ - 1);

            // Update the shared front index so the producer knows we've
            // consumed the slot
            __atomic_store_n(p_req_front, req_front, memory_order_release);
        } else {
            // No data: busy-poll for a defined period before deciding to sleep
            if (++poll_count < MAX_POLL_COUNT) {
                continue;
            }
            poll_count = 0;

            /*
             * Dekker's Algorithm Step 1: Signal intention to sleep.
             * The producer will check this flag to decide whether to write to
             * eventfd.
             */
            __atomic_store_n(p_need_wakeup, 1, memory_order_relaxed);

            /*
             * Dekker's Algorithm Step 2: Full System Memory Barrier.
             * This prevents the Store (need_wakeup=1) from being reordered with
             * the Load (req_rear), which is critical to avoid missing a late
             * update.
             */
            __atomic_thread_fence(memory_order_seq_cst);

            /*
             * Dekker's Algorithm Step 3: Final re-check.
             * Check if the producer added a request between our last check and
             * setting the flag.
             */
            req_rear = __atomic_load_n(p_req_rear, memory_order_relaxed);
            if (req_front == req_rear) {
                // Confirmed empty: exit loop and enter epoll_wait in the caller
                break;
            }

            // Race detected: producer added work, so clear flag and keep
            // processing
            __atomic_store_n(p_need_wakeup, 0, memory_order_relaxed);
        }
    }

    return proc_count;
}

/**
 * @brief Main event loop for handling VirtIO requests and system signals
 *
 * This function implements the core event loop for the VirtIO backend. It sets
 * up signal handling and event notification mechanisms, then enters an infinite
 * loop waiting for either VirtIO kick events from the kernel or termination
 * signals.
 *
 * Key functionality includes:
 * - Blocks SIGINT and SIGTERM signals for synchronous handling via signalfd
 * - Creates and configures epoll instance to monitor both signalfd and eventfd
 * - Initializes the shared memory state to indicate readiness for notifications
 * - Processes incoming events using epoll_wait() with infinite timeout
 * - Handles termination signals gracefully by cleaning up resources
 * - Delegates VirtIO request processing to consume_pending_requests()
 *
 * @return void
 *
 * @note The function runs indefinitely until a termination signal is received
 */
void handle_virtio_requests(void) {
    // Block signals to handle them synchronously via signalfd
    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    if (sigprocmask(SIG_BLOCK, &mask, NULL) == -1) {
        log_error("Failed to set sigprocmask");
        return;
    }

    sfd = signalfd(-1, &mask, SFD_NONBLOCK | SFD_CLOEXEC);
    if (sfd == -1) {
        log_error("Failed to create signalfd");
        return;
    }

    epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (epoll_fd == -1) {
        log_error("Failed to create epoll instance");
        close(sfd);
        return;
    }

    // Register signalfd for termination handling
    struct epoll_event ev;
    ev.events = EPOLLIN;
    ev.data.fd = sfd;
    if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, sfd, &ev) == -1) {
        log_error("epoll_ctl failed for signalfd");
        close(sfd);
        close(epoll_fd);
        return;
    }

    // Register eventfd for kernel-to-user VirtIO kicks
    ev.events = EPOLLIN;
    ev.data.fd = efd;
    if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, efd, &ev) == -1) {
        log_error("epoll_ctl failed for eventfd");
        close(sfd);
        close(epoll_fd);
        return;
    }

    log_info("virtio request handler loop started.");
    int signal_count = 0, proc_count = 0;
    struct epoll_event events[16];

#ifdef LOONGARCH64
    /*
     * Drain requests that may have arrived before the event loop became ready.
     * consume_pending_requests() also performs the sleep handshake: it sets
     * need_wakeup, issues a full barrier, and rechecks the ring before
     * returning.
     */
    proc_count += consume_pending_requests();
#else
    // Preserve the existing initialization behavior on other architectures.
    __atomic_store_n(&virtio_bridge->need_wakeup, 1, memory_order_relaxed);
#endif

    while (true) {
        log_debug("signal_count is %d, proc_count is %d", signal_count,
                  proc_count);

        // Wait indefinitely for a signal or a kernel kick
        int nfds = epoll_wait(epoll_fd, events, 16, -1);
        ++signal_count;
        if (nfds == -1) {
            if (errno == EINTR)
                continue;
            log_error("epoll_wait failed");
            virtio_close();
            break;
        }

        for (int i = 0; i < nfds; ++i) {
            if (events[i].data.fd == sfd) {
                struct signalfd_siginfo fdsi;
                if (read(sfd, &fdsi, sizeof(fdsi)) == sizeof(fdsi)) {
                    log_info("Received termination signal %d. Exiting...",
                             fdsi.ssi_signo);
                    virtio_close();
                    return;
                }
            } else if (events[i].data.fd == efd) {
                uint64_t u;
                // Clear the eventfd counter to acknowledge the notification
                if (read(efd, &u, sizeof(uint64_t)) != sizeof(uint64_t)) {
                    continue;
                }

                // Process all pending requests until the ring is empty
                proc_count += consume_pending_requests();
            }
        }
    }
}

int virtio_init() {
    // The higher log level is, the faster virtio-blk will be.
    int err;

    // Define signal set and add all signals to the set
    sigset_t block_mask;
    sigfillset(&block_mask);
    pthread_sigmask(SIG_BLOCK, &block_mask, NULL);

    // Set process name
    prctl(PR_SET_NAME, "hvisor-virtio", 0, 0, 0);

    log_info("hvisor init");
    ko_fd = open(HVISOR_DEVICE, O_RDWR);
    if (ko_fd < 0) {
        log_error("open hvisor failed");
        exit(1);
    }
    // ioctl for init virtio
    // Communicate with hvisor kernel module
    err = ioctl(ko_fd, HVISOR_INIT_VIRTIO);
    if (err) {
        log_error("ioctl failed, err code is %d", err);
        close(ko_fd);
        exit(1);
    }

    // create eventfd
    efd = eventfd(0, EFD_NONBLOCK | EFD_CLOEXEC);
    if (efd < 0) {
        log_error("eventfd failed, errno is %d", errno);
        close(ko_fd);
        exit(1);
    }
    if (ioctl(ko_fd, HVISOR_SET_EVENTFD, efd) < 0) {
        log_error("ioctl HVISOR_SET_EVENTFD failed, errno is %d", errno);
        close(ko_fd);
        close(efd);
        exit(1);
    }

    // mmap: create shared memory
    // Map the virtio_bridge set by the kernel module to this space
    virtio_bridge = (struct virtio_bridge *)mmap(
        NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, ko_fd, 0);
    if (virtio_bridge == (void *)-1) {
        log_error("mmap failed");
        goto unmap;
    }

    // Initialize event_monitor used by console and net devices
    initialize_event_monitor();
    log_info("hvisor init okay!");
    return 0;
unmap:
    munmap((void *)virtio_bridge, MMAP_SIZE);
    return -1;
}

static int parse_virtio_device_type(const char *type,
                                    VirtioDeviceType *dev_type) {
    if (strcmp(type, "blk") == 0)
        *dev_type = VirtioTBlock;
    else if (strcmp(type, "net") == 0)
        *dev_type = VirtioTNet;
    else if (strcmp(type, "console") == 0)
        *dev_type = VirtioTConsole;
    else if (strcmp(type, "gpu") == 0)
        *dev_type = VirtioTGPU;
    else if (strcmp(type, "scmi") == 0)
        *dev_type = VirtioTSCMI;
    else {
        log_error("unknown device type %s", type);
        return -1;
    }
    return 0;
}

static void free_virtio_device_config(VirtioDeviceConfig *cfg) {
    if (cfg->params && cfg->config_ops && cfg->config_ops->free)
        cfg->config_ops->free(cfg->params);
    cfg->params = NULL;
}

static void free_virtio_config(VirtioConfig *cfg) {
    for (size_t zi = 0; zi < cfg->zone_num; zi++)
        for (size_t di = 0; di < cfg->zones[zi].device_num; di++)
            free_virtio_device_config(&cfg->zones[zi].devices[di]);
}

static int parse_virtio_device_config(const cJSON *device_json,
                                      uint32_t zone_id,
                                      VirtioDeviceConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cfg->zone_id = zone_id;

    cJSON *status = SAFE_CJSON_GET_OBJECT_ITEM(device_json, "status");
    if (!cJSON_IsString(status)) {
        log_error("failed to parse device status");
        return -1;
    }
    if (strcmp(status->valuestring, "disable") == 0)
        return 0;
    if (strcmp(status->valuestring, "enable") != 0) {
        log_error("unknown device status %s", status->valuestring);
        return -1;
    }
    cfg->enabled = true;

    cJSON *type = SAFE_CJSON_GET_OBJECT_ITEM(device_json, "type");
    if (!cJSON_IsString(type) ||
        parse_virtio_device_type(type->valuestring, &cfg->type) != 0)
        return -1;

    if (parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "addr"),
                       &cfg->addr) != 0 ||
        parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "len"),
                       &cfg->len) != 0 ||
        parse_json_u32(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "irq"),
                       &cfg->irq) != 0 ||
        cfg->addr == 0 || cfg->len == 0 || cfg->irq == 0) {
        log_error("failed to parse addr, len, or irq");
        return -1;
    }

    cfg->config_ops = lookup_config_ops(cfg->type);
    if (!cfg->config_ops || !cfg->config_ops->parse ||
        cfg->config_ops->parse(device_json, &cfg->params) != 0) {
        log_error("failed to parse parameters for %s",
                  virtio_device_type_to_string(cfg->type));
        free_virtio_device_config(cfg);
        return -1;
    }
    return 0;
}

static int parse_virtio_memory_region_config(const cJSON *mem_json,
                                             VirtioMemoryRegionConfig *cfg) {
    return parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "zone0_ipa"),
                          &cfg->zone0_ipa) != 0 ||
                   parse_json_u64(
                       SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "zonex_ipa"),
                       &cfg->zonex_ipa) != 0 ||
                   parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "size"),
                                  &cfg->size) != 0
               ? -1
               : 0;
}

static int parse_virtio_zone_config(const cJSON *zone_json,
                                    VirtioZoneConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cJSON *id = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "id");
    if (!id)
        id = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "zone_id");
    if (parse_json_u32(id, &cfg->zone_id) != 0 || cfg->zone_id >= MAX_ZONES)
        return -1;

    cJSON *memory = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "memory_region");
    int memory_num = SAFE_CJSON_GET_ARRAY_SIZE(memory);
    if (memory_num < 0 || memory_num > CONFIG_MAX_MEMORY_REGIONS)
        return -1;
    cfg->memory_region_num = (size_t)memory_num;
    for (size_t i = 0; i < cfg->memory_region_num; i++)
        if (parse_virtio_memory_region_config(
                SAFE_CJSON_GET_ARRAY_ITEM(memory, i),
                &cfg->memory_regions[i]) != 0)
            return -1;

    cJSON *devices = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "devices");
    int device_num = SAFE_CJSON_GET_ARRAY_SIZE(devices);
    if (device_num < 0 || device_num > MAX_DEVS)
        return -1;
    cfg->device_num = (size_t)device_num;
    for (size_t i = 0; i < cfg->device_num; i++) {
        if (parse_virtio_device_config(SAFE_CJSON_GET_ARRAY_ITEM(devices, i),
                                       cfg->zone_id, &cfg->devices[i]) != 0)
            return -1;
    }
    return 0;
}

static int parse_virtio_config(const cJSON *root, VirtioConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cJSON *zones = cJSON_GetObjectItemCaseSensitive(root, "zones");
    if (!zones) {
        cfg->zone_num = 1;
        return parse_virtio_zone_config(root, &cfg->zones[0]);
    }

    int zone_num = SAFE_CJSON_GET_ARRAY_SIZE(zones);
    if (zone_num < 0 || zone_num > MAX_ZONES)
        return -1;
    cfg->zone_num = (size_t)zone_num;
    for (size_t i = 0; i < cfg->zone_num; i++)
        if (parse_virtio_zone_config(SAFE_CJSON_GET_ARRAY_ITEM(zones, i),
                                     &cfg->zones[i]) != 0)
            return -1;
    return 0;
}

static uint64_t range_end(uint64_t base, uint64_t len) {
    return UINT64_MAX - base < len ? UINT64_MAX : base + len;
}

static bool ranges_overlap(uint64_t base_a, uint64_t len_a, uint64_t base_b,
                           uint64_t len_b) {
    if (len_a == 0 || len_b == 0)
        return false;
    return base_a < range_end(base_b, len_b) &&
           base_b < range_end(base_a, len_a);
}

static bool same_memory_region(const VirtioMemoryRegionConfig *a,
                               const VirtioMemoryRegionConfig *b) {
    return a->zone0_ipa == b->zone0_ipa && a->zonex_ipa == b->zonex_ipa &&
           a->size == b->size;
}

static int find_zone_memory_region_locked(uint32_t zone_id,
                                          const VirtioMemoryRegionConfig *mem) {
    struct zone_mem *z = &zone_mem[zone_id];
    for (size_t i = 0; i < z->num_regions; i++) {
        struct zone_mem_region *existing = &z->regions[i];
        VirtioMemoryRegionConfig current = {
            .zone0_ipa = existing->zone0_ipa,
            .zonex_ipa = existing->zonex_ipa,
            .size = existing->mem_size,
        };
        if (same_memory_region(&current, mem))
            return (int)i;
        if (ranges_overlap(mem->zone0_ipa, mem->size, current.zone0_ipa,
                           current.size) ||
            ranges_overlap(mem->zonex_ipa, mem->size, current.zonex_ipa,
                           current.size))
            return -2;
    }
    return -1;
}

static int validate_virtio_config_devices(const VirtioConfig *cfg) {
    size_t total = 0;
    pthread_mutex_lock(&VDEV_MUTEX);
    total = (size_t)vdevs_num;
    for (size_t zi = 0; zi < cfg->zone_num; zi++) {
        const VirtioZoneConfig *zone = &cfg->zones[zi];
        for (size_t di = 0; di < zone->device_num; di++) {
            const VirtioDeviceConfig *dev = &zone->devices[di];
            if (!dev->enabled)
                continue;
            if (++total > MAX_DEVS)
                goto fail;
            for (int i = 0; i < vdevs_num; i++)
                if (vdevs[i]->zone_id == dev->zone_id &&
                    ranges_overlap(dev->addr, dev->len, vdevs[i]->base_addr,
                                   vdevs[i]->len))
                    goto fail;
            for (size_t pzi = 0; pzi <= zi; pzi++) {
                const VirtioZoneConfig *previous = &cfg->zones[pzi];
                size_t limit = pzi == zi ? di : previous->device_num;
                for (size_t pdi = 0; pdi < limit; pdi++) {
                    const VirtioDeviceConfig *previous_dev =
                        &previous->devices[pdi];
                    if (previous_dev->enabled &&
                        previous_dev->zone_id == dev->zone_id &&
                        ranges_overlap(dev->addr, dev->len, previous_dev->addr,
                                       previous_dev->len))
                        goto fail;
                }
            }
        }
    }
    pthread_mutex_unlock(&VDEV_MUTEX);
    return 0;
fail:
    pthread_mutex_unlock(&VDEV_MUTEX);
    log_error("virtio device limit or MMIO range conflict");
    return -1;
}

static int mmap_virtio_zone_memory(const VirtioZoneConfig *cfg) {
    for (size_t i = 0; i < cfg->memory_region_num; i++) {
        const VirtioMemoryRegionConfig *mem = &cfg->memory_regions[i];
        pthread_mutex_lock(&ZONE_MEM_MUTEX);
        int found = find_zone_memory_region_locked(cfg->zone_id, mem);
        if (found >= 0) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            continue;
        }
        if (found == -2 || mem->size == 0 ||
            zone_mem[cfg->zone_id].num_regions >= CONFIG_MAX_MEMORY_REGIONS) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            return -1;
        }
        size_t slot = zone_mem[cfg->zone_id].num_regions;
        pthread_mutex_unlock(&ZONE_MEM_MUTEX);

        void *virt_addr = mmap(NULL, mem->size, PROT_READ | PROT_WRITE,
                               MAP_SHARED, ko_fd, (off_t)mem->zone0_ipa);
        if (virt_addr == MAP_FAILED)
            return -1;

        pthread_mutex_lock(&ZONE_MEM_MUTEX);
        if (slot != zone_mem[cfg->zone_id].num_regions) {
            pthread_mutex_unlock(&ZONE_MEM_MUTEX);
            munmap(virt_addr, mem->size);
            return -1;
        }
        zone_mem[cfg->zone_id].regions[slot] = (struct zone_mem_region){
            .virt_addr = (uintptr_t)virt_addr,
            .zone0_ipa = mem->zone0_ipa,
            .zonex_ipa = mem->zonex_ipa,
            .mem_size = mem->size,
        };
        zone_mem[cfg->zone_id].num_regions++;
        pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    }
    return 0;
}

static int create_virtio_config_devices(const VirtioConfig *cfg) {
    VirtIODevice *new_devs[MAX_DEVS] = {0};
    size_t count = 0;
    for (size_t zi = 0; zi < cfg->zone_num; zi++) {
        const VirtioZoneConfig *zone = &cfg->zones[zi];
        for (size_t di = 0; di < zone->device_num; di++) {
            const VirtioDeviceConfig *dev = &zone->devices[di];
            if (!dev->enabled)
                continue;
            VirtIODevice *vdev = create_virtio_device_unpublished(
                dev->type, dev->zone_id, dev->addr, dev->len, dev->irq,
                dev->params);
            if (!vdev || count == MAX_DEVS) {
                if (vdev)
                    destroy_unpublished_virtio_device(vdev);
                for (size_t i = 0; i < count; i++)
                    destroy_unpublished_virtio_device(new_devs[i]);
                return -1;
            }
            new_devs[count++] = vdev;
        }
    }
    if (publish_virtio_devices(new_devs, count) != 0) {
        for (size_t i = 0; i < count; i++)
            destroy_unpublished_virtio_device(new_devs[i]);
        return -1;
    }
    return 0;
}

static ssize_t read_full(int fd, void *buf, size_t len) {
    size_t off = 0;
    while (off < len) {
        ssize_t ret = read(fd, (char *)buf + off, len - off);
        if (ret == 0)
            return (ssize_t)off;
        if (ret < 0) {
            if (errno == EINTR)
                continue;
            return -1;
        }
        off += (size_t)ret;
    }
    return (ssize_t)off;
}

static ssize_t write_full(int fd, const void *buf, size_t len) {
    size_t off = 0;
    while (off < len) {
        ssize_t ret = write(fd, (const char *)buf + off, len - off);
        if (ret < 0) {
            if (errno == EINTR)
                continue;
            return -1;
        }
        if (ret == 0)
            return (ssize_t)off;
        off += (size_t)ret;
    }
    return (ssize_t)off;
}

int virtio_start_from_json(char *json_path) {
    uint64_t file_size = 0;
    char *buffer = read_file(json_path, &file_size);
    if (!buffer)
        return -1;
    buffer[file_size] = '\0';
    cJSON *root = SAFE_CJSON_PARSE(buffer);
    VirtioConfig config;
    int err = -1;
    if (!root)
        goto out;
    if (parse_virtio_config(root, &config) != 0 ||
        validate_virtio_config_devices(&config) != 0)
        goto out_config;
    for (size_t i = 0; i < config.zone_num; i++)
        if (mmap_virtio_zone_memory(&config.zones[i]) != 0)
            goto out_config;
    if (create_virtio_config_devices(&config) != 0)
        goto out_config;
    err = 0;
out_config:
    free_virtio_config(&config);
    cJSON_Delete(root);
out:
    free(buffer);
    return err;
}

static int virtio_add_from_json(const char *json_path) {
    int err = virtio_start_from_json((char *)json_path);
    if (err)
        return err;
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; i++)
        virtio_bridge->mmio_addrs[i] = vdevs[i]->base_addr;
    pthread_mutex_unlock(&VDEV_MUTEX);
    write_barrier();
    virtio_bridge->mmio_avail = 1;
    write_barrier();
    return 0;
}

static void fill_control_response(VirtioControlResponse *resp, int status,
                                  const char *message) {
    memset(resp, 0, sizeof(*resp));
    resp->status = status;
    snprintf(resp->message, sizeof(resp->message), "%s", message);
}

static void handle_control_client(int client_fd) {
    VirtioControlRequest req;
    VirtioControlResponse resp;
    memset(&req, 0, sizeof(req));
    if (read_full(client_fd, &req, sizeof(req)) != sizeof(req))
        return;
    req.op[sizeof(req.op) - 1] = '\0';
    req.json_path[sizeof(req.json_path) - 1] = '\0';
    if (strcmp(req.op, VIRTIO_CTRL_OP_ADD) != 0)
        fill_control_response(&resp, -1,
                              "unsupported virtio control operation");
    else if (virtio_add_from_json(req.json_path) != 0)
        fill_control_response(&resp, -1, "virtio add failed");
    else
        fill_control_response(&resp, 0, "virtio add succeeded");
    if (atomic_load(&ctrl_running))
        write_full(client_fd, &resp, sizeof(resp));
}

static void *virtio_control_loop(void *arg) {
    (void)arg;
    while (atomic_load(&ctrl_running)) {
        int client_fd = accept(ctrl_fd, NULL, NULL);
        if (client_fd < 0) {
            if (errno == EINTR)
                continue;
            if (!atomic_load(&ctrl_running) &&
                (errno == EINVAL || errno == EBADF || errno == ENOTSOCK))
                break;
            if (!atomic_load(&ctrl_running))
                break;
            log_error("accept virtio control client failed, errno is %d",
                      errno);
            continue;
        }
        pthread_mutex_lock(&ctrl_client_lock);
        if (!atomic_load(&ctrl_running)) {
            pthread_mutex_unlock(&ctrl_client_lock);
            shutdown(client_fd, SHUT_RDWR);
            close(client_fd);
            break;
        }
        ctrl_client_fd = client_fd;
        pthread_mutex_unlock(&ctrl_client_lock);
        handle_control_client(client_fd);
        pthread_mutex_lock(&ctrl_client_lock);
        if (ctrl_client_fd == client_fd)
            ctrl_client_fd = -1;
        pthread_mutex_unlock(&ctrl_client_lock);
        close(client_fd);
    }
    return NULL;
}

static int start_virtio_control_server(void) {
    struct sockaddr_un addr;
    unlink(VIRTIO_CTRL_SOCKET_PATH);
    ctrl_fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (ctrl_fd < 0)
        return -1;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s",
             VIRTIO_CTRL_SOCKET_PATH);
    if (bind(ctrl_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0 ||
        listen(ctrl_fd, 8) != 0) {
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    pthread_mutex_lock(&ctrl_client_lock);
    ctrl_client_fd = -1;
    pthread_mutex_unlock(&ctrl_client_lock);
    atomic_store(&ctrl_running, true);
    if (pthread_create(&ctrl_tid, NULL, virtio_control_loop, NULL) != 0) {
        atomic_store(&ctrl_running, false);
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    ctrl_thread_started = true;
    return 0;
}

static void stop_virtio_control_server(void) {
    if (!ctrl_thread_started && ctrl_fd < 0)
        return;
    atomic_store(&ctrl_running, false);
    if (ctrl_fd >= 0)
        shutdown(ctrl_fd, SHUT_RDWR);
    pthread_mutex_lock(&ctrl_client_lock);
    int client_fd = ctrl_client_fd;
    pthread_mutex_unlock(&ctrl_client_lock);
    if (client_fd >= 0)
        shutdown(client_fd, SHUT_RDWR);
    if (ctrl_thread_started) {
        pthread_join(ctrl_tid, NULL);
        ctrl_thread_started = false;
    }
    pthread_mutex_lock(&ctrl_client_lock);
    ctrl_client_fd = -1;
    pthread_mutex_unlock(&ctrl_client_lock);
    if (ctrl_fd >= 0) {
        close(ctrl_fd);
        ctrl_fd = -1;
    }
    unlink(VIRTIO_CTRL_SOCKET_PATH);
}

int virtio_start(int argc, char *argv[]) {
    int err;
    if (argc < 4)
        return -1;
    err = virtio_init();
    if (err)
        return err;
    err = virtio_start_from_json(argv[3]);
    if (err)
        goto err_out;
    if (start_virtio_control_server() != 0) {
        err = -1;
        goto err_out;
    }
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; i++)
        virtio_bridge->mmio_addrs[i] = vdevs[i]->base_addr;
    pthread_mutex_unlock(&VDEV_MUTEX);
    write_barrier();
    virtio_bridge->mmio_avail = 1;
    write_barrier();
    handle_virtio_requests();
    return 0;
err_out:
    virtio_close();
    return err;
}

int virtio_add(int argc, char *argv[]) {
    VirtioControlRequest req;
    VirtioControlResponse resp;
    struct sockaddr_un addr;
    char path[PATH_MAX];
    if (argc < 4 || !realpath(argv[3], path))
        return -1;
    int fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0)
        return -1;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s",
             VIRTIO_CTRL_SOCKET_PATH);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return -1;
    }
    memset(&req, 0, sizeof(req));
    snprintf(req.op, sizeof(req.op), "%s", VIRTIO_CTRL_OP_ADD);
    snprintf(req.json_path, sizeof(req.json_path), "%s", path);
    if (write_full(fd, &req, sizeof(req)) != sizeof(req) ||
        read_full(fd, &resp, sizeof(resp)) != sizeof(resp)) {
        close(fd);
        return -1;
    }
    close(fd);
    printf("%s\n", resp.message);
    return resp.status;
}
