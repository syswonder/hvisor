// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 */
#include <asm/byteorder.h>
#include <linux/fs.h>
#include <linux/io.h>
#include <linux/kernel.h>
#include <linux/moduleparam.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/virtio_blk.h>

#include "virtio_pci_blk_backend.h"
#include "virtio_pci_bridge.h"

#define HVISOR_VIRTIO_BLK_SECTOR_SIZE 512
#define HVISOR_VIRTIO_BLK_DEVICE_ID "hvisor-virtblk"
#define HVISOR_VIRTIO_BLK_MAX_SEGS 512

struct hvisor_virtio_pci_blk_dev {
    struct file *image;
    struct virtq_desc *chain;
    u64 capacity_sectors;
    char id[VIRTIO_BLK_ID_BYTES];
};

static struct hvisor_virtio_pci_blk_dev virtio_pci_blk_devs[MAX_VIRTPCI_DEV];
static char virtio_pci_blk_image_path[256];
static struct kparam_string virtio_pci_blk_image_path_param = {
    .maxlen = sizeof(virtio_pci_blk_image_path),
    .string = virtio_pci_blk_image_path,
};

static void virtio_pci_blk_sanitize_path(char *path) {
    size_t len;

    strim(path);
    len = strlen(path);
    while (len > 0 && (path[len - 1] == '\n' || path[len - 1] == '\r')) {
        path[len - 1] = '\0';
        len--;
    }
}

static int virtio_pci_blk_image_path_set(const char *val,
                                         const struct kernel_param *kp) {
    int ret = param_set_copystring(val, kp);

    if (ret == 0)
        virtio_pci_blk_sanitize_path(virtio_pci_blk_image_path);
    return ret;
}

static const struct kernel_param_ops virtio_pci_blk_image_path_ops = {
    .set = virtio_pci_blk_image_path_set,
    .get = param_get_string,
};

module_param_cb(virtio_pci_blk_image_path, &virtio_pci_blk_image_path_ops,
                &virtio_pci_blk_image_path_param, 0644);
MODULE_PARM_DESC(virtio_pci_blk_image_path,
                 "Path to the file image used as virtio-pci blk backend");

static void virtio_pci_blk_complete(struct virtq_used *used, __u16 used_slot,
                                    __u32 head, __u32 total_len) {
    used->ring[used_slot].id = head;
    used->ring[used_slot].len = total_len;
}

static int virtio_pci_blk_collect_chain(struct virtq_desc *desc, __u16 head,
                                        __u16 queue_size,
                                        struct virtq_desc *chain,
                                        int *chain_len) {
    __u16 idx = head;
    int len = 0;

    while (1) {
        if (idx >= queue_size || len >= HVISOR_VIRTIO_BLK_MAX_SEGS + 2)
            return -EINVAL;
        if (desc[idx].flags & VRING_DESC_F_INDIRECT)
            return -EOPNOTSUPP;

        chain[len++] = desc[idx];
        if (!(desc[idx].flags & VRING_DESC_F_NEXT))
            break;
        idx = desc[idx].next;
    }

    *chain_len = len;
    return 0;
}

static int virtio_pci_blk_rw(struct hvisor_virtio_pci_blk_dev *blk,
                             struct virtq_desc *chain, int chain_len,
                             struct virtio_blk_outhdr *hdr, u32 *total_len) {
    loff_t offset =
        (loff_t)le64_to_cpu(hdr->sector) * HVISOR_VIRTIO_BLK_SECTOR_SIZE;
    u32 done = 0;
    int i;
    bool is_write = le32_to_cpu(hdr->type) == VIRTIO_BLK_T_OUT;

    for (i = 1; i < chain_len - 1; i++) {
        void *buf = memremap(chain[i].addr, chain[i].len, MEMREMAP_WB);
        ssize_t io_len;

        if (buf == NULL)
            return -ENOMEM;
        if (offset + chain[i].len >
            blk->capacity_sectors * HVISOR_VIRTIO_BLK_SECTOR_SIZE) {
            memunmap(buf);
            return -EIO;
        }
        if (is_write)
            io_len = kernel_write(blk->image, buf, chain[i].len, &offset);
        else
            io_len = kernel_read(blk->image, buf, chain[i].len, &offset);
        memunmap(buf);
        if (io_len < 0)
            return (int)io_len;
        if (io_len != chain[i].len)
            return -EIO;
        done += io_len;
    }

    *total_len = done;
    return 0;
}

static int virtio_pci_blk_handle_one(struct hvisor_virtio_pci_blk_dev *blk,
                                     struct virtq_desc *desc,
                                     struct virtq_avail *avail,
                                     struct virtq_used *used, __u16 avail_slot,
                                     __u16 queue_size) {
    struct virtq_desc *chain = blk->chain;
    struct virtio_blk_outhdr *hdr;
    __u16 head = avail->ring[avail_slot];
    __u16 used_slot = used->idx % queue_size;
    u32 total_len = 1;
    u8 status = VIRTIO_BLK_S_OK;
    void *status_buf;
    int chain_len;
    int ret;

    ret =
        virtio_pci_blk_collect_chain(desc, head, queue_size, chain, &chain_len);
    if (ret)
        return ret;
    if (chain_len < 2 || chain[0].len < sizeof(*hdr) ||
        chain[chain_len - 1].len < 1)
        return -EINVAL;

    hdr = memremap(chain[0].addr, sizeof(*hdr), MEMREMAP_WB);
    if (hdr == NULL)
        return -ENOMEM;

    switch (le32_to_cpu(hdr->type)) {
    case VIRTIO_BLK_T_IN:
    case VIRTIO_BLK_T_OUT:
        ret = virtio_pci_blk_rw(blk, chain, chain_len, hdr, &total_len);
        break;
    case VIRTIO_BLK_T_GET_ID: {
        void *id_buf;
        size_t copy_len;

        if (chain_len != 3) {
            ret = -EINVAL;
            break;
        }
        id_buf = memremap(chain[1].addr, chain[1].len, MEMREMAP_WB);
        if (id_buf == NULL) {
            ret = -ENOMEM;
            break;
        }
        copy_len = min_t(size_t, chain[1].len, sizeof(blk->id));
        memset(id_buf, 0, chain[1].len);
        memcpy(id_buf, blk->id, copy_len);
        memunmap(id_buf);
        total_len = copy_len + 1;
        ret = 0;
        break;
    }
    default:
        ret = -EOPNOTSUPP;
        break;
    }

    memunmap(hdr);

    if (ret == -EOPNOTSUPP)
        status = VIRTIO_BLK_S_UNSUPP;
    else if (ret)
        status = VIRTIO_BLK_S_IOERR;

    status_buf = memremap(chain[chain_len - 1].addr, 1, MEMREMAP_WB);
    if (status_buf == NULL)
        return -ENOMEM;
    *(u8 *)status_buf = status;
    memunmap(status_buf);

    if (ret && ret != -EOPNOTSUPP)
        total_len = 1;
    virtio_pci_blk_complete(used, used_slot, head, total_len);
    used->idx++;
    return 0;
}

int hvisor_virtio_pci_blk_init_dev(__u16 dev_id) {
    struct hvisor_virtio_pci_blk_dev *blk;
    struct file *image;
    loff_t file_size;

    if (dev_id >= MAX_VIRTPCI_DEV)
        return -EINVAL;

    blk = &virtio_pci_blk_devs[dev_id];
    if (blk->image != NULL)
        return 0;
    virtio_pci_blk_sanitize_path(virtio_pci_blk_image_path);
    if (virtio_pci_blk_image_path[0] == '\0') {
        pr_err("virtio blk image path is empty\n");
        return -EINVAL;
    }

    image = filp_open(virtio_pci_blk_image_path, O_RDWR | O_LARGEFILE, 0);
    if (IS_ERR(image)) {
        pr_err("failed to open virtio blk image %s, err=%ld\n",
               virtio_pci_blk_image_path, PTR_ERR(image));
        return PTR_ERR(image);
    }
    blk->image = image;

    file_size = i_size_read(file_inode(blk->image));
    if (file_size <= 0 || file_size % HVISOR_VIRTIO_BLK_SECTOR_SIZE != 0) {
        pr_err("virtio blk image size is invalid: %lld\n",
               (long long)file_size);
        filp_close(blk->image, NULL);
        blk->image = NULL;
        return -EINVAL;
    }

    blk->chain = kcalloc(HVISOR_VIRTIO_BLK_MAX_SEGS + 2, sizeof(*blk->chain),
                         GFP_KERNEL);
    if (!blk->chain) {
        filp_close(blk->image, NULL);
        blk->image = NULL;
        return -ENOMEM;
    }
    blk->capacity_sectors = file_size / HVISOR_VIRTIO_BLK_SECTOR_SIZE;
    memset(blk->id, 0, sizeof(blk->id));
    strscpy(blk->id, HVISOR_VIRTIO_BLK_DEVICE_ID, sizeof(blk->id));
    return 0;
}

void hvisor_virtio_pci_blk_exit(void) {
    int i;

    for (i = 0; i < MAX_VIRTPCI_DEV; i++) {
        if (virtio_pci_blk_devs[i].image != NULL) {
            filp_close(virtio_pci_blk_devs[i].image, NULL);
            virtio_pci_blk_devs[i].image = NULL;
        }
        kfree(virtio_pci_blk_devs[i].chain);
        virtio_pci_blk_devs[i].chain = NULL;
        virtio_pci_blk_devs[i].capacity_sectors = 0;
        memset(virtio_pci_blk_devs[i].id, 0, sizeof(virtio_pci_blk_devs[i].id));
    }
}

void hvisor_virtio_pci_blk_handler(__u16 dev_id, struct virtqueue_info *vq,
                                   int queue_id) {
    struct hvisor_virtio_pci_blk_dev *blk;
    struct virtq_desc *desc;
    struct virtq_avail *avail;
    struct virtq_used *used;
    __u16 queue_size;
    __u16 used_idx;

    if (dev_id >= MAX_VIRTPCI_DEV)
        return;

    blk = &virtio_pci_blk_devs[dev_id];
    if (blk->image == NULL) {
        pr_err("virtio blk device %u is not initialized\n", dev_id);
        return;
    }

    queue_size = (__u16)vq->queue_size;
    desc = (struct virtq_desc *)(uintptr_t)vq->desc_area;
    avail = (struct virtq_avail *)(uintptr_t)vq->avail_area;
    used = (struct virtq_used *)(uintptr_t)vq->used_area;
    if (queue_size == 0 || desc == NULL || avail == NULL || used == NULL) {
        pr_err("virtio blk queue %d is invalid\n", queue_id);
        return;
    }

    used_idx = used->idx;
    while (used_idx != avail->idx) {
        __u16 slot = used_idx % queue_size;

        if (virtio_pci_blk_handle_one(blk, desc, avail, used, slot, queue_size))
            break;
        used_idx = used->idx;
    }
}
