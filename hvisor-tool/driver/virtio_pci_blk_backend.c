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

#include "virtio_pci_backend.h"
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
                             const struct virtio_pci_owner *owner,
                             struct virtio_blk_outhdr *hdr, u32 *total_len) {
    const u64 sector = le64_to_cpu(hdr->sector);
    const u64 capacity = blk->capacity_sectors * HVISOR_VIRTIO_BLK_SECTOR_SIZE;
    const bool is_write = le32_to_cpu(hdr->type) == VIRTIO_BLK_T_OUT;
    u32 done = 0;
    loff_t offset;
    int i;

    if (sector >= blk->capacity_sectors)
        return -EIO;
    offset = sector * HVISOR_VIRTIO_BLK_SECTOR_SIZE;
    for (i = 1; i < chain_len - 1; i++) {
        void *buf;
        ssize_t io_len;

        if (chain[i].len > capacity - offset ||
            (!!(chain[i].flags & VRING_DESC_F_WRITE) == is_write))
            return -EIO;
        buf = hvisor_virtio_pci_map_ram(owner, chain[i].addr, chain[i].len,
                                        chain[i].len);
        if (!buf)
            return -EFAULT;
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

static void virtio_pci_blk_handle_one(struct hvisor_virtio_pci_blk_dev *blk,
                                      const struct virtio_pci_owner *owner,
                                      struct virtq_desc *desc,
                                      struct virtq_avail *avail,
                                      struct virtq_used *used, __u16 avail_slot,
                                      __u16 queue_size) {
    struct virtq_desc *chain = blk->chain;
    struct virtio_blk_outhdr hdr;
    void *header_buf;
    __u16 head = avail->ring[avail_slot];
    __u16 used_slot = used->idx % queue_size;
    u32 total_len = 0;
    u8 status = VIRTIO_BLK_S_OK;
    void *status_buf;
    int chain_len;
    int ret;

    ret =
        virtio_pci_blk_collect_chain(desc, head, queue_size, chain, &chain_len);
    /* An invalid chain may have no safe status byte to update. */
    if (ret || chain_len < 2 ||
        !(chain[chain_len - 1].flags & VRING_DESC_F_WRITE) ||
        chain[chain_len - 1].len < 1)
        goto complete;

    status_buf =
        hvisor_virtio_pci_map_ram(owner, chain[chain_len - 1].addr,
                                  chain[chain_len - 1].len, sizeof(status));
    if (!status_buf)
        goto complete;

    ret = -EFAULT;
    if (chain[0].len < sizeof(hdr) || (chain[0].flags & VRING_DESC_F_WRITE))
        goto write_status;
    header_buf = hvisor_virtio_pci_map_ram(owner, chain[0].addr, chain[0].len,
                                           sizeof(hdr));
    if (!header_buf)
        goto write_status;
    memcpy(&hdr, header_buf, sizeof(hdr));
    memunmap(header_buf);

    switch (le32_to_cpu(hdr.type)) {
    case VIRTIO_BLK_T_IN:
    case VIRTIO_BLK_T_OUT:
        ret = virtio_pci_blk_rw(blk, chain, chain_len, owner, &hdr, &total_len);
        break;
    case VIRTIO_BLK_T_GET_ID: {
        void *id_buf;
        size_t copy_len;

        if (chain_len != 3 || !(chain[1].flags & VRING_DESC_F_WRITE)) {
            ret = -EINVAL;
            break;
        }
        id_buf = hvisor_virtio_pci_map_ram(owner, chain[1].addr, chain[1].len,
                                           chain[1].len);
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

write_status:
    if (ret == -EOPNOTSUPP)
        status = VIRTIO_BLK_S_UNSUPP;
    else if (ret)
        status = VIRTIO_BLK_S_IOERR;

    *(u8 *)status_buf = status;
    memunmap(status_buf);
    if (ret)
        total_len = 1;
complete:
    virtio_pci_blk_complete(used, used_slot, head, total_len);
    smp_store_release(&used->idx, used->idx + 1);
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

void hvisor_virtio_pci_blk_close_dev(__u16 dev_id) {
    struct hvisor_virtio_pci_blk_dev *blk = &virtio_pci_blk_devs[dev_id];

    if (blk->image)
        filp_close(blk->image, NULL);
    kfree(blk->chain);
    memset(blk, 0, sizeof(*blk));
}

void hvisor_virtio_pci_blk_handler(__u16 dev_id,
                                   const struct virtio_pci_owner *owner,
                                   struct virtqueue_info *vq, int queue_id) {
    struct hvisor_virtio_pci_blk_dev *blk;
    struct virtq_desc *desc;
    struct virtq_avail *avail;
    struct virtq_used *used;
    __u16 queue_size;
    unsigned int i;
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

    used_idx = READ_ONCE(used->idx);
    for (i = 0; i < queue_size && used_idx != smp_load_acquire(&avail->idx);
         i++) {
        __u16 slot = used_idx % queue_size;

        virtio_pci_blk_handle_one(blk, owner, desc, avail, used, slot,
                                  queue_size);
        used_idx = READ_ONCE(used->idx);
    }
}
