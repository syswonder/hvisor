// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 */
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
#include <linux/string.h>
#include <linux/uaccess.h>
#include <linux/virtio_ring.h>
#include <linux/workqueue.h>

#include "hvisor.h"
#include "virtio_pci_backend.h"
#include "virtio_pci_blk_backend.h"

struct virtio_pci_dev {
    __u16 num_of_vq;
    struct virtqueue_info vqs[MAX_VQ];
    void (*data_req_handler)(struct virtqueue_info *, int queue_id);
};

static struct virtio_pci_bridge *virtio_pci_bridge;
static DEFINE_MUTEX(virtio_pci_lock);

static int virtio_pci_irq_config = -1;
static int virtio_pci_irq_data = -1;
static struct virtio_pci_dev virtpci_dev_list[MAX_VIRTPCI_DEV];
static int nxt_dev_idx;
static bool virtio_pci_userspace_blk;
static bool virtio_pci_userspace_rng;

module_param_named(virtio_pci_userspace_blk, virtio_pci_userspace_blk, bool,
                   0644);
MODULE_PARM_DESC(virtio_pci_userspace_blk,
                 "Delegate virtio-pci blk backend handling to userspace");
module_param_named(virtio_pci_userspace_rng, virtio_pci_userspace_rng, bool,
                   0644);
MODULE_PARM_DESC(virtio_pci_userspace_rng,
                 "Delegate virtio-pci rng backend handling to userspace");

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

static int
create_virtio_pci_dev(__u16 num_of_vq, struct virtqueue_info *vqs, __u16 dtype,
                      void (*handler)(struct virtqueue_info *, int q_id)) {
    int i = 0;
    int ret = -ENOMEM;
    struct virtio_pci_dev *birth;

    int limit = virtio_pci_userspace_devices() ? VIRTIO_PCI_USERSPACE_DEV_BASE
                                               : MAX_VIRTPCI_DEV;

    if (nxt_dev_idx >= limit) {
        pr_err("virtio PCI kernel backend has no free device slots\n");
        return -ENOSPC;
    }
    if (num_of_vq == 0 || num_of_vq > MAX_VQ) {
        pr_err("num_of_vq > MAX_VQ there are too much virtqueu in the virtio "
               "device\n");
        return -EINVAL;
    }

    birth = &virtpci_dev_list[nxt_dev_idx];
    memset(birth, 0, sizeof(*birth));
    birth->num_of_vq = num_of_vq;
    while (i < num_of_vq) {
        __u64 queue_size = vqs[i].queue_size;

        if (!is_power_of_2(queue_size) || queue_size > U16_MAX) {
            pr_err("invalid virtio pci queue size\n");
            ret = -EINVAL;
            goto err_unmap;
        }

        birth->vqs[i].desc_area = (__u64)memremap(
            vqs[i].desc_area, sizeof(struct virtq_desc) * queue_size,
            MEMREMAP_WB);
        birth->vqs[i].avail_area = (__u64)memremap(
            vqs[i].avail_area, virtio_avail_ring_size(queue_size), MEMREMAP_WB);
        birth->vqs[i].used_area = (__u64)memremap(
            vqs[i].used_area, virtio_used_ring_size(queue_size), MEMREMAP_WB);
        if (!birth->vqs[i].desc_area || !birth->vqs[i].avail_area ||
            !birth->vqs[i].used_area) {
            pr_err("virtio pci queue area memremap failed\n");
            ret = -ENOMEM;
            goto err_unmap;
        }
        birth->vqs[i].queue_size = queue_size;
        i++;
    }

    birth->data_req_handler = handler;
    if (dtype == VIRTIO_PCI_BLK) {
        ret = hvisor_virtio_pci_blk_init_dev(nxt_dev_idx);
        if (ret)
            goto err_unmap;
    }
    return nxt_dev_idx++;

err_unmap:
    if (i >= num_of_vq)
        i = num_of_vq - 1;
    while (i >= 0) {
        if (birth->vqs[i].desc_area)
            memunmap((void *)(uintptr_t)birth->vqs[i].desc_area);
        if (birth->vqs[i].avail_area)
            memunmap((void *)(uintptr_t)birth->vqs[i].avail_area);
        if (birth->vqs[i].used_area)
            memunmap((void *)(uintptr_t)birth->vqs[i].used_area);
        birth->vqs[i].desc_area = 0;
        birth->vqs[i].avail_area = 0;
        birth->vqs[i].used_area = 0;
        birth->vqs[i].queue_size = 0;
        i--;
    }
    return ret;
}

static void virtio_rng_handler(struct virtqueue_info *vq, int queue_id) {
    __u16 queue_size = (__u16)vq->queue_size;
    struct virtq_desc *desc = (struct virtq_desc *)(uintptr_t)vq->desc_area;
    struct virtq_avail *avail = (struct virtq_avail *)(uintptr_t)vq->avail_area;
    struct virtq_used *used = (struct virtq_used *)(uintptr_t)vq->used_area;
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

    while (used_idx != avail_idx) {
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

            buffer = memremap(desc_item.addr, desc_item.len, MEMREMAP_WB);
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

static void virtio_pci_process_config(void) {
    while (!virtio_pci_config_req_empty()) {
        struct virtio_pci_config_req *peek_req = virtio_pci_peek_config_req();

        if (virtio_pci_is_userspace_dtype(peek_req->info.dtype) ||
            virtio_pci_config_res_full())
            break;

        __u32 status = 0;
        __u16 dev_idx = 0;
        void (*handler)(struct virtqueue_info *, int) = NULL;
        struct virtio_pci_config_req req = *peek_req;
        struct virtio_pci_config_info *info = &req.info;
        int ret;

        switch (info->dtype) {
        case VIRTIO_PCI_BLK:
            handler = NULL;
            break;
        case VIRTIO_PCI_RNG:
            handler = virtio_rng_handler;
            break;
        default:
            status = EOPNOTSUPP;
            break;
        }

        if (!status) {
            ret = create_virtio_pci_dev(info->num_of_queues, info->vqs,
                                        info->dtype, handler);
            if (ret < 0)
                status = -ret;
            else
                dev_idx = (__u16)ret;
        }

        virtio_pci_push_config_res(req.request_id, status, dev_idx);
        virtio_pci_advance_config_req();
    }
}

static void virtio_pci_process_data(void) {

    while (!virtio_pci_data_req_empty()) {
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
        struct virtqueue_info *vq;

        if (dev_idx >= nxt_dev_idx) {
            pr_err("the dev_idx given by hvisor is invaild!\n");
            continue;
        }

        dev = &virtpci_dev_list[dev_idx];
        if (queue_id >= dev->num_of_vq) {
            pr_err("the queue_id given by hvisor is invaild!\n");
            continue;
        }
        vq = &dev->vqs[queue_id];

        if (dev->data_req_handler != NULL)
            dev->data_req_handler(vq, queue_id);
        else
            hvisor_virtio_pci_blk_handler(info->dev_id, vq, queue_id);
        virtio_pci_write_hypercall_info(VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE,
                                        cpu_id, req.request_id, 0);
        hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_DOORBELL, 0);
    }
}

static void virtio_pci_config_workfn(struct work_struct *work) {
    virtio_pci_process_config();
}

static void virtio_pci_data_workfn(struct work_struct *work) {
    mutex_lock(&virtio_pci_lock);
    virtio_pci_process_data();
    mutex_unlock(&virtio_pci_lock);
}

static DECLARE_WORK(virtio_pci_config_work, virtio_pci_config_workfn);
static DECLARE_WORK(virtio_pci_data_work, virtio_pci_data_workfn);

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
    static_assert(sizeof(struct virtio_pci_bridge) <= VIRTIO_PCI_BRIDGE_SIZE);
    virtio_pci_bridge = (void *)get_zeroed_page(GFP_KERNEL);
    if (!virtio_pci_bridge) {
        err = -ENOMEM;
        goto put_nodes;
    }
    SetPageReserved(virt_to_page(virtio_pci_bridge));

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
    int dev_idx, queue;

    if (!virtio_pci_bridge)
        return;

    free_irq(virtio_pci_irq_data, NULL);
    free_irq(virtio_pci_irq_config, NULL);
    cancel_work_sync(&virtio_pci_config_work);
    cancel_work_sync(&virtio_pci_data_work);
    hvisor_call(HVISOR_HC_VIRTIO_PCI, VIRTIO_PCI_HC_DEINIT,
                __pa(virtio_pci_bridge));

    hvisor_virtio_pci_blk_exit();
    for (dev_idx = 0; dev_idx < nxt_dev_idx; dev_idx++) {
        struct virtio_pci_dev *dev = &virtpci_dev_list[dev_idx];

        for (queue = 0; queue < dev->num_of_vq; queue++) {
            struct virtqueue_info *vq = &dev->vqs[queue];

            memunmap((void *)(uintptr_t)vq->desc_area);
            memunmap((void *)(uintptr_t)vq->avail_area);
            memunmap((void *)(uintptr_t)vq->used_area);
        }
    }
    ClearPageReserved(virt_to_page(virtio_pci_bridge));
    free_page((unsigned long)virtio_pci_bridge);
    virtio_pci_bridge = NULL;
}
