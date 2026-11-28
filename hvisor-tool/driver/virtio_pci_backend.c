// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 */
#include <linux/capability.h>
#include <linux/gfp.h>
#include <linux/interrupt.h>
#include <linux/io.h>
#include <linux/kernel.h>
#include <linux/mm.h>
#include <linux/moduleparam.h>
#include <linux/mutex.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <linux/random.h>
#include <linux/slab.h>
#include <linux/spinlock.h>
#include <linux/string.h>
#include <linux/uaccess.h>
#include <linux/virtio_ring.h>
#include <linux/wait.h>
#include <linux/workqueue.h>

#include "hvisor.h"
#include "virtio_pci_backend.h"
#include "virtio_pci_blk_backend.h"

struct virtio_pci_dev {
    struct virtio_pci_owner owner;
    __u16 dtype;
    __u16 num_of_vq;
    struct virtqueue_info vqs[MAX_VQ];
};

static struct virtio_pci_bridge *virtio_pci_bridge;
static DEFINE_MUTEX(virtio_pci_lock);
static DEFINE_SPINLOCK(virtio_pci_io_lock);
static DECLARE_WAIT_QUEUE_HEAD(virtio_pci_io_drained);
static unsigned int virtio_pci_active_io;
static bool virtio_pci_io_paused;
static __u32 virtio_pci_cleanup_epoch;

static int virtio_pci_irq_config = -1;
static int virtio_pci_irq_data = -1;
static struct virtio_pci_dev virtpci_dev_list[MAX_VIRTPCI_DEV];
static bool virtio_pci_userspace_blk;
static bool virtio_pci_userspace_rng;

module_param_named(virtio_pci_userspace_blk, virtio_pci_userspace_blk, bool,
                   0444);
MODULE_PARM_DESC(virtio_pci_userspace_blk,
                 "Delegate virtio-pci blk backend handling to userspace");
module_param_named(virtio_pci_userspace_rng, virtio_pci_userspace_rng, bool,
                   0444);
MODULE_PARM_DESC(virtio_pci_userspace_rng,
                 "Delegate virtio-pci rng backend handling to userspace");

static bool virtio_pci_begin_io(__u32 *epoch) {
    bool allowed;

    spin_lock(&virtio_pci_io_lock);
    allowed = !virtio_pci_io_paused;
    if (allowed) {
        virtio_pci_active_io++;
        if (epoch)
            *epoch = virtio_pci_cleanup_epoch;
    }
    spin_unlock(&virtio_pci_io_lock);
    return allowed;
}

static void virtio_pci_end_io(void) {
    bool drained;

    spin_lock(&virtio_pci_io_lock);
    virtio_pci_active_io--;
    drained = virtio_pci_io_paused && !virtio_pci_active_io;
    spin_unlock(&virtio_pci_io_lock);
    if (drained)
        wake_up(&virtio_pci_io_drained);
}

int hvisor_virtio_pci_begin_io(struct virtio_pci_file *ctx, __u32 __user *arg) {
    __u32 epoch;
    int ret = 0;

    if (!virtio_pci_bridge)
        return -ENODEV;
    if (!capable(CAP_SYS_ADMIN))
        return -EPERM;

    mutex_lock(&ctx->lock);
    if (ctx->io_active) {
        ret = -EALREADY;
    } else if (!virtio_pci_begin_io(&epoch)) {
        ret = -EBUSY;
    } else if (copy_to_user(arg, &epoch, sizeof(epoch))) {
        virtio_pci_end_io();
        ret = -EFAULT;
    } else {
        ctx->io_active = true;
    }
    mutex_unlock(&ctx->lock);
    return ret;
}

void hvisor_virtio_pci_end_io(struct virtio_pci_file *ctx) {
    mutex_lock(&ctx->lock);
    if (ctx->io_active) {
        ctx->io_active = false;
        virtio_pci_end_io();
    }
    mutex_unlock(&ctx->lock);
}

int hvisor_virtio_pci_owner_active(struct virtio_pci_owner __user *arg) {
    struct virtio_pci_owner owner;
    __u64 identity;

    if (!virtio_pci_bridge)
        return -ENODEV;
    if (copy_from_user(&owner, arg, sizeof(owner)))
        return -EFAULT;

    identity = ((__u64)owner.zone_id << 32) | owner.zone_generation;
    return hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_OWNER_ACTIVE,
                       identity);
}

static bool virtio_pci_is_userspace_dtype(__u16 dtype) {
    switch (dtype) {
    case VIRTIO_PCI_BLK:
        return virtio_pci_userspace_blk;
    case VIRTIO_PCI_RNG:
        return virtio_pci_userspace_rng;
    default:
        return false;
    }
}

static unsigned int virtio_pci_userspace_devices(void) {
    return (virtio_pci_userspace_blk ? VIRTIO_PCI_USERSPACE_BLK : 0) |
           (virtio_pci_userspace_rng ? VIRTIO_PCI_USERSPACE_RNG : 0);
}

static void virtio_pci_write_hypercall_info(__u16 op, __u32 target_cpu,
                                            __u32 request_id, __u32 status) {
    struct virtio_pci_hypercall_info *info = &virtio_pci_bridge->hypercall_info;

    WRITE_ONCE(info->version, VIRTIO_PCI_HYPERCALL_VERSION);
    WRITE_ONCE(info->op, op);
    WRITE_ONCE(info->target_cpu, target_cpu);
    WRITE_ONCE(info->request_id, request_id);
    WRITE_ONCE(info->status, status);
    smp_wmb();
}

static size_t virtio_avail_ring_size(__u64 queue_size) {
    return sizeof(__u16) * 2 + sizeof(__u16) * queue_size;
}

static size_t virtio_used_ring_size(__u64 queue_size) {
    return sizeof(__u16) * 2 + sizeof(struct virtq_used_elem) * queue_size;
}

static bool virtio_pci_config_req_empty(void) {
    return smp_load_acquire(&virtio_pci_bridge->config_req_front) ==
           smp_load_acquire(&virtio_pci_bridge->config_req_rear);
}

static bool virtio_pci_config_res_full(void) {
    __u32 front = READ_ONCE(virtio_pci_bridge->config_res_front);
    __u32 rear = READ_ONCE(virtio_pci_bridge->config_res_rear);

    smp_rmb();
    return ((rear + 1) & (MAX_PCI_CONFIG_RES - 1)) == front;
}

static bool virtio_pci_data_req_empty(void) {
    return smp_load_acquire(&virtio_pci_bridge->data_req_front) ==
           smp_load_acquire(&virtio_pci_bridge->data_req_rear);
}

static void virtio_pci_advance_config_req(void) {
    __u32 front = READ_ONCE(virtio_pci_bridge->config_req_front);

    smp_store_release(&virtio_pci_bridge->config_req_front,
                      (front + 1) & (MAX_PCI_CONFIG_REQ - 1));
}

static void virtio_pci_push_config_res(__u32 request_id, __u32 status,
                                       __u16 dev_id) {
    __u32 rear;
    struct virtio_pci_config_res *res;

    if (virtio_pci_config_res_full()) {
        pr_err("virtio pci config response queue is full\n");
        return;
    }

    rear = READ_ONCE(virtio_pci_bridge->config_res_rear);
    res = &virtio_pci_bridge->config_res_list[rear];
    res->request_id = request_id;
    res->status = status;
    res->dev_id = dev_id;
    res->padding = 0;
    smp_wmb();
    WRITE_ONCE(virtio_pci_bridge->config_res_rear,
               (rear + 1) & (MAX_PCI_CONFIG_RES - 1));
}

static struct virtio_pci_data_req virtio_pci_pop_data_req(void) {
    __u32 front = READ_ONCE(virtio_pci_bridge->data_req_front);
    struct virtio_pci_data_req req = virtio_pci_bridge->data_req_list[front];

    smp_store_release(&virtio_pci_bridge->data_req_front,
                      (front + 1) & (MAX_PCI_DATA_REQ - 1));
    return req;
}

static struct virtio_pci_config_req *virtio_pci_peek_config_req(void) {
    __u32 front = READ_ONCE(virtio_pci_bridge->config_req_front);

    return &virtio_pci_bridge->config_req_list[front];
}

static struct virtio_pci_data_req *virtio_pci_peek_data_req(void) {
    __u32 front = READ_ONCE(virtio_pci_bridge->data_req_front);

    return &virtio_pci_bridge->data_req_list[front];
}

static int virtio_pci_resolve_ram(struct virtio_pci_ram_request *query) {
    struct virtio_pci_ram_request *shared;
    int ret;

    shared = kmemdup(query, sizeof(*shared), GFP_KERNEL);
    if (!shared)
        return -ENOMEM;

    ret = hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_RESOLVE_RAM,
                      __pa(shared));
    if (!ret)
        query->pa = shared->pa;
    kfree(shared);
    return ret;
}

int hvisor_virtio_pci_resolve_ram(struct virtio_pci_ram_request __user *arg) {
    struct virtio_pci_ram_request query;
    int ret;

    if (!virtio_pci_bridge)
        return -ENODEV;
    if (copy_from_user(&query, arg, sizeof(query)))
        return -EFAULT;
    ret = virtio_pci_resolve_ram(&query);
    if (ret)
        return ret;
    return copy_to_user(arg, &query, sizeof(query)) ? -EFAULT : 0;
}

void *hvisor_virtio_pci_map_ram(const struct virtio_pci_owner *owner, __u64 ipa,
                                size_t dma_len, size_t access_len) {
    struct virtio_pci_ram_request query = {
        .ipa = ipa,
        .len = dma_len,
        .owner = *owner,
    };

    if (!access_len || access_len > dma_len || virtio_pci_resolve_ram(&query))
        return NULL;
    return memremap(query.pa, access_len, MEMREMAP_WB);
}

static bool virtio_pci_queue_valid(const struct virtqueue_info *queue) {
    return is_power_of_2(queue->queue_size) && queue->queue_size <= U16_MAX &&
           IS_ALIGNED(queue->desc_area, 16) &&
           IS_ALIGNED(queue->avail_area, 2) && IS_ALIGNED(queue->used_area, 4);
}

static void virtio_pci_unmap_queue(struct virtqueue_info *queue) {
    if (queue->desc_area)
        memunmap((void *)(uintptr_t)queue->desc_area);
    if (queue->avail_area)
        memunmap((void *)(uintptr_t)queue->avail_area);
    if (queue->used_area)
        memunmap((void *)(uintptr_t)queue->used_area);
}

static int virtio_pci_map_queue(const struct virtio_pci_owner *owner,
                                const struct virtqueue_info *guest,
                                struct virtqueue_info *queue) {
    __u64 size = guest->queue_size;

    if (!virtio_pci_queue_valid(guest))
        return -EINVAL;

    size_t desc_len = sizeof(struct virtq_desc) * size;
    size_t avail_len = virtio_avail_ring_size(size);
    size_t used_len = virtio_used_ring_size(size);

    *queue = (struct virtqueue_info){
        .queue_size = size,
        .desc_area = (__u64)hvisor_virtio_pci_map_ram(owner, guest->desc_area,
                                                      desc_len, desc_len),
        .avail_area = (__u64)hvisor_virtio_pci_map_ram(owner, guest->avail_area,
                                                       avail_len, avail_len),
        .used_area = (__u64)hvisor_virtio_pci_map_ram(owner, guest->used_area,
                                                      used_len, used_len),
    };
    if (!queue->desc_area || !queue->avail_area || !queue->used_area) {
        virtio_pci_unmap_queue(queue);
        return -EFAULT;
    }
    return 0;
}

static int create_virtio_pci_dev(const struct virtio_pci_config_req *request) {
    const struct virtio_pci_config_info *info = &request->info;
    struct virtio_pci_dev *dev;
    struct virtio_pci_owner owner = {
        .zone_id = info->zone_id,
        .zone_generation = request->zone_generation,
    };
    int limit = virtio_pci_userspace_devices() ? VIRTIO_PCI_USERSPACE_DEV_BASE
                                               : MAX_VIRTPCI_DEV;
    int i, ret, slot;

    if (!info->num_of_queues || info->num_of_queues > MAX_VQ)
        return -EINVAL;

    for (slot = 0; slot < limit; slot++) {
        if (!virtpci_dev_list[slot].num_of_vq)
            break;
    }
    if (slot == limit)
        return -ENOSPC;

    for (i = 0; i < info->num_of_queues; i++) {
        struct virtqueue_info mapped;

        ret = virtio_pci_map_queue(&owner, &info->vqs[i], &mapped);
        if (ret)
            return ret;
        virtio_pci_unmap_queue(&mapped);
    }
    if (info->dtype == VIRTIO_PCI_BLK) {
        ret = hvisor_virtio_pci_blk_init_dev(slot);
        if (ret)
            return ret;
    }

    dev = &virtpci_dev_list[slot];
    dev->owner = owner;
    dev->dtype = info->dtype;
    dev->num_of_vq = info->num_of_queues;
    memcpy(dev->vqs, info->vqs, info->num_of_queues * sizeof(info->vqs[0]));
    return slot;
}

static void virtio_rng_handler(const struct virtio_pci_owner *owner,
                               struct virtqueue_info *vq) {
    __u16 queue_size = (__u16)vq->queue_size;
    struct virtq_desc *desc = (struct virtq_desc *)(uintptr_t)vq->desc_area;
    struct virtq_avail *avail = (struct virtq_avail *)(uintptr_t)vq->avail_area;
    struct virtq_used *used = (struct virtq_used *)(uintptr_t)vq->used_area;
    unsigned int i;
    __u16 used_idx;
    __u16 avail_idx;

    if (desc == NULL || avail == NULL || used == NULL) {
        pr_err("desc:%p avail:%p used:%p\n", desc, avail, used);
        return;
    }
    if (queue_size == 0) {
        pr_err("virtio rng queue size is zero\n");
        return;
    }

    used_idx = READ_ONCE(used->idx);
    smp_rmb();
    avail_idx = READ_ONCE(avail->idx);

    for (i = 0; i < queue_size && used_idx != avail_idx; i++) {
        __u16 slot = used_idx % queue_size;
        __u16 head = READ_ONCE(avail->ring[slot]);
        __u16 idx = head;
        __u16 seen = 0;
        __u32 total_len = 0;

        if (head >= queue_size) {
            pr_err("virtio rng invalid desc head %u, queue size %u\n", head,
                   queue_size);
            goto complete;
        }

        for (;;) {
            struct virtq_desc desc_item;
            void *buffer;

            if (seen++ >= queue_size || idx >= queue_size) {
                pr_err("virtio rng invalid descriptor chain\n");
                total_len = 0;
                break;
            }

            desc_item = desc[idx];
            if (desc_item.flags & VRING_DESC_F_INDIRECT) {
                pr_err("virtio rng indirect descriptors are unsupported\n");
                total_len = 0;
                break;
            }
            if ((desc_item.flags & VRING_DESC_F_WRITE) == 0) {
                pr_err("virtio rng descriptor %u is not writable\n", idx);
                total_len = 0;
                break;
            }

            buffer = hvisor_virtio_pci_map_ram(owner, desc_item.addr,
                                               desc_item.len, desc_item.len);
            if (buffer == NULL) {
                pr_err("virtio rng failed to map desc %u addr=%llx len=%u\n",
                       idx, desc_item.addr, desc_item.len);
                total_len = 0;
                break;
            }

            get_random_bytes(buffer, desc_item.len);
            memunmap(buffer);
            total_len += desc_item.len;

            if ((desc_item.flags & VRING_DESC_F_NEXT) == 0)
                break;
            idx = desc_item.next;
        }

    complete:
        used->ring[slot].id = head;
        used->ring[slot].len = total_len;
        smp_wmb();
        used_idx++;
        WRITE_ONCE(used->idx, used_idx);
        smp_mb();
        avail_idx = READ_ONCE(avail->idx);
    }
}

static bool virtio_pci_process_config(void) {
    unsigned int i;

    for (i = 0; i < MAX_PCI_CONFIG_REQ && !virtio_pci_config_req_empty(); i++) {
        struct virtio_pci_config_req *peek_req = virtio_pci_peek_config_req();

        if (virtio_pci_is_userspace_dtype(peek_req->info.dtype) ||
            virtio_pci_config_res_full())
            break;

        __u32 status = 0;
        __u16 dev_idx = 0;
        struct virtio_pci_config_req req = *peek_req;
        struct virtio_pci_config_info *info = &req.info;
        int ret;

        if (info->dtype != VIRTIO_PCI_BLK && info->dtype != VIRTIO_PCI_RNG) {
            status = EOPNOTSUPP;
        } else {
            ret = create_virtio_pci_dev(&req);
            if (ret < 0)
                status = -ret;
            else
                dev_idx = (__u16)ret;
        }

        virtio_pci_push_config_res(req.request_id, status, dev_idx);
        virtio_pci_advance_config_req();
    }
    return i == MAX_PCI_CONFIG_REQ;
}

static bool virtio_pci_process_data(void) {
    unsigned int i;

    for (i = 0; i < MAX_PCI_DATA_REQ && !virtio_pci_data_req_empty(); i++) {
        struct virtio_pci_data_req *peek_req = virtio_pci_peek_data_req();

        if (virtio_pci_userspace_devices() &&
            peek_req->info.dev_id >= VIRTIO_PCI_USERSPACE_DEV_BASE)
            break;

        struct virtio_pci_data_req req = virtio_pci_pop_data_req();
        struct virtio_pci_data_info *info = &req.info;
        __u64 dev_idx = info->dev_id;
        __u64 queue_id = info->queue_id;
        __u32 cpu_id = info->cpu_id;
        struct virtio_pci_dev *dev;
        struct virtqueue_info queue;
        int ret = -EINVAL;

        if (dev_idx >= MAX_VIRTPCI_DEV)
            goto complete;
        dev = &virtpci_dev_list[dev_idx];
        if (dev->owner.zone_generation != req.zone_generation ||
            queue_id >= dev->num_of_vq)
            goto complete;

        ret = virtio_pci_map_queue(&dev->owner, &dev->vqs[queue_id], &queue);
        if (ret)
            goto complete;
        if (dev->dtype == VIRTIO_PCI_RNG)
            virtio_rng_handler(&dev->owner, &queue);
        else
            hvisor_virtio_pci_blk_handler(info->dev_id, &dev->owner, &queue,
                                          queue_id);
        virtio_pci_unmap_queue(&queue);
    complete:
        virtio_pci_write_hypercall_info(VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE,
                                        cpu_id, req.request_id, -ret);
        hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_DOORBELL, 0);
    }
    return i == MAX_PCI_DATA_REQ;
}

static void virtio_pci_workfn(struct work_struct *work, bool (*process)(void)) {
    bool more;

    if (!virtio_pci_begin_io(NULL))
        return;
    mutex_lock(&virtio_pci_lock);
    more = process();
    mutex_unlock(&virtio_pci_lock);
    virtio_pci_end_io();

    if (more)
        schedule_work(work);
}

static void virtio_pci_config_workfn(struct work_struct *work) {
    virtio_pci_workfn(work, virtio_pci_process_config);
}

static void virtio_pci_data_workfn(struct work_struct *work) {
    virtio_pci_workfn(work, virtio_pci_process_data);
}

static DECLARE_WORK(virtio_pci_config_work, virtio_pci_config_workfn);
static DECLARE_WORK(virtio_pci_data_work, virtio_pci_data_workfn);

static void virtio_pci_close_dev(unsigned int slot) {
    struct virtio_pci_dev *dev = &virtpci_dev_list[slot];

    if (dev->dtype == VIRTIO_PCI_BLK)
        hvisor_virtio_pci_blk_close_dev(slot);
    memset(dev, 0, sizeof(*dev));
}

static bool virtio_pci_io_idle(void) {
    bool idle;

    spin_lock(&virtio_pci_io_lock);
    idle = !virtio_pci_active_io;
    spin_unlock(&virtio_pci_io_lock);
    return idle;
}

int hvisor_virtio_pci_shutdown_zone(unsigned long zone_id) {
    unsigned int slot;
    int ret;

    if (!virtio_pci_bridge || !zone_id)
        return hvisor_call(HVISOR_HC_SHUTDOWN_ZONE, zone_id, 0);

    /* Stop admitting DMA before entering EL2 so none can run after it
     * releases the zone's RAM. Wait here in Linux, where workers can sleep.
     */
    spin_lock(&virtio_pci_io_lock);
    virtio_pci_io_paused = true;
    spin_unlock(&virtio_pci_io_lock);
    ret = wait_event_killable(virtio_pci_io_drained, virtio_pci_io_idle());
    if (!ret)
        ret = hvisor_call(HVISOR_HC_SHUTDOWN_ZONE, zone_id,
                          HVISOR_VIRTIO_PCI_SHUTDOWN_DRAINED);
    if (!ret) {
        for (slot = 0; slot < MAX_VIRTPCI_DEV; slot++) {
            if (virtpci_dev_list[slot].num_of_vq &&
                virtpci_dev_list[slot].owner.zone_id == zone_id)
                virtio_pci_close_dev(slot);
        }
    }

    spin_lock(&virtio_pci_io_lock);
    if (!ret)
        virtio_pci_cleanup_epoch++;
    virtio_pci_io_paused = false;
    spin_unlock(&virtio_pci_io_lock);
    schedule_work(&virtio_pci_config_work);
    schedule_work(&virtio_pci_data_work);
    return ret;
}

static irqreturn_t virtio_pci_irq_handler(int irq, void *dev_id) {
    if (irq == virtio_pci_irq_config)
        schedule_work(&virtio_pci_config_work);
    else
        schedule_work(&virtio_pci_data_work);
    return IRQ_HANDLED;
}

int hvisor_virtio_pci_get_info(struct hvisor_virtio_pci_info __user *arg) {
    struct hvisor_virtio_pci_info info = {};

    if (!virtio_pci_bridge)
        return -ENODEV;
    info.pfn = virt_to_phys(virtio_pci_bridge) >> PAGE_SHIFT;
    info.userspace_devices = virtio_pci_userspace_devices();
    if (copy_to_user(arg, &info, sizeof(info)))
        return -EFAULT;
    return 0;
}

int hvisor_virtio_pci_notify(struct virtio_pci_hypercall_info __user *arg) {
    struct virtio_pci_hypercall_info info;
    int ret = 0;

    if (!virtio_pci_bridge || !virtio_pci_userspace_devices())
        return -ENODEV;
    if (copy_from_user(&info, arg, sizeof(info)))
        return -EFAULT;
    if (info.version != VIRTIO_PCI_HYPERCALL_VERSION ||
        (info.op != VIRTIO_PCI_HC_OP_CONFIG_REQ_COMPLETE &&
         info.op != VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE))
        return -EINVAL;

    if (info.op == VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE) {
        mutex_lock(&virtio_pci_lock);
        virtio_pci_write_hypercall_info(info.op, info.target_cpu,
                                        info.request_id, info.status);
        ret = hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_DOORBELL, 0);
        mutex_unlock(&virtio_pci_lock);
    }

    /* A kernel-owned request may now be at the head of this queue. */
    if (info.op == VIRTIO_PCI_HC_OP_CONFIG_REQ_COMPLETE)
        schedule_work(&virtio_pci_config_work);
    else
        schedule_work(&virtio_pci_data_work);
    return ret;
}

int hvisor_virtio_pci_init(void) {
    struct device_node *config_node, *data_node;
    int err = 0;

    if (!IS_ENABLED(CONFIG_ARM64))
        return 0;

    config_node = of_find_node_by_path("/hvisor_virtio_pci_config");
    data_node = of_find_node_by_path("/hvisor_virtio_pci_data");
    if (!config_node || !data_node)
        goto put_nodes;

    virtio_pci_irq_config = of_irq_get(config_node, 0);
    virtio_pci_irq_data = of_irq_get(data_node, 0);
    if (virtio_pci_irq_config <= 0 || virtio_pci_irq_data <= 0) {
        err = virtio_pci_irq_config <= 0 ? virtio_pci_irq_config
                                         : virtio_pci_irq_data;
        if (!err)
            err = -EINVAL;
        goto put_nodes;
    }
    if (hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_GET_VERSION, 0) !=
        VIRTIO_PCI_HYPERCALL_VERSION) {
        pr_err("incompatible VirtIO PCI bridge version\n");
        err = -EPROTONOSUPPORT;
        goto put_nodes;
    }
    static_assert(sizeof(struct virtio_pci_bridge) <= VIRTIO_PCI_BRIDGE_SIZE);
    virtio_pci_bridge = (void *)get_zeroed_page(GFP_KERNEL);
    if (!virtio_pci_bridge) {
        err = -ENOMEM;
        goto put_nodes;
    }
    SetPageReserved(virt_to_page(virtio_pci_bridge));
    virtio_pci_bridge->hypercall_info.version = VIRTIO_PCI_HYPERCALL_VERSION;

    err = request_irq(virtio_pci_irq_config, virtio_pci_irq_handler, 0,
                      "hvisor_virtio_pci_config", NULL);
    if (err)
        goto free_bridge;
    err = request_irq(virtio_pci_irq_data, virtio_pci_irq_handler, 0,
                      "hvisor_virtio_pci_data", NULL);
    if (err)
        goto free_config_irq;

    err = hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_INIT,
                      __pa(virtio_pci_bridge));
    if (!err)
        goto put_nodes;
    free_irq(virtio_pci_irq_data, NULL);
free_config_irq:
    free_irq(virtio_pci_irq_config, NULL);
free_bridge:
    cancel_work_sync(&virtio_pci_config_work);
    cancel_work_sync(&virtio_pci_data_work);
    ClearPageReserved(virt_to_page(virtio_pci_bridge));
    free_page((unsigned long)virtio_pci_bridge);
    virtio_pci_bridge = NULL;
put_nodes:
    of_node_put(config_node);
    of_node_put(data_node);
    return err;
}

void hvisor_virtio_pci_exit(void) {
    unsigned int slot;

    if (!virtio_pci_bridge)
        return;

    free_irq(virtio_pci_irq_data, NULL);
    free_irq(virtio_pci_irq_config, NULL);
    cancel_work_sync(&virtio_pci_config_work);
    cancel_work_sync(&virtio_pci_data_work);
    hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_DEINIT,
                __pa(virtio_pci_bridge));

    for (slot = 0; slot < MAX_VIRTPCI_DEV; slot++)
        virtio_pci_close_dev(slot);
    ClearPageReserved(virt_to_page(virtio_pci_bridge));
    free_page((unsigned long)virtio_pci_bridge);
    virtio_pci_bridge = NULL;
}
