// SPDX-License-Identifier: GPL-2.0-only
#include <errno.h>
#include <fcntl.h>
#include <linux/virtio_blk.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include "virtio_pci_userspace.h"

#define HVISOR_VIRTIO_BLK_SECTOR_SIZE 512
#define HVISOR_VIRTIO_BLK_DEVICE_ID "hvisor-virtblk"
#define HVISOR_VIRTIO_BLK_MAX_SEGS 512
#define HVISOR_VIRTIO_PCI_BLK_IMAGE_PATH                                       \
    "/sys/module/hvisor/parameters/virtio_pci_blk_image_path"

static void userspace_blk_sanitize_path(char *path) {
    size_t start = 0;
    size_t end;

    while (path[start] == ' ' || path[start] == '\t' || path[start] == '\n' ||
           path[start] == '\r')
        start++;
    if (start != 0)
        memmove(path, path + start, strlen(path + start) + 1);

    end = strlen(path);
    while (end > 0 && (path[end - 1] == ' ' || path[end - 1] == '\t' ||
                       path[end - 1] == '\n' || path[end - 1] == '\r')) {
        path[end - 1] = '\0';
        end--;
    }
}

static int userspace_blk_open_image(UserspacePciBlkDev *blk) {
    char path[256];
    ssize_t len;
    struct stat st;
    int fd;

    fd = open(HVISOR_VIRTIO_PCI_BLK_IMAGE_PATH, O_RDONLY);
    if (fd < 0)
        return -errno;

    len = read(fd, path, sizeof(path) - 1);
    close(fd);
    if (len <= 0)
        return len == 0 ? -ENOENT : -errno;

    path[len] = '\0';
    userspace_blk_sanitize_path(path);
    if (path[0] == '\0')
        return -ENOENT;

    fd = open(path, O_RDWR);
    if (fd < 0)
        return -errno;
    if (fstat(fd, &st) != 0) {
        int err = errno;

        close(fd);
        return -err;
    }
    if (st.st_size <= 0 || (st.st_size % HVISOR_VIRTIO_BLK_SECTOR_SIZE) != 0) {
        close(fd);
        return -EINVAL;
    }

    blk->image_fd = fd;
    blk->capacity_sectors =
        (uint64_t)st.st_size / HVISOR_VIRTIO_BLK_SECTOR_SIZE;
    memset(blk->id, 0, sizeof(blk->id));
    strncpy(blk->id, HVISOR_VIRTIO_BLK_DEVICE_ID, sizeof(blk->id) - 1);
    return 0;
}

static int userspace_blk_collect_chain(volatile struct virtq_desc *desc,
                                       __u16 head, __u16 queue_size,
                                       struct virtq_desc *chain,
                                       int *chain_len) {
    __u16 idx = head;
    int len = 0;

    while (1) {
        if (idx >= queue_size || len >= HVISOR_VIRTIO_BLK_MAX_SEGS + 2)
            return -EINVAL;
        if (desc[idx].flags & VRING_DESC_F_INDIRECT)
            return -EOPNOTSUPP;

        chain[len].addr = desc[idx].addr;
        chain[len].len = desc[idx].len;
        chain[len].flags = desc[idx].flags;
        chain[len].next = desc[idx].next;
        len++;

        if ((desc[idx].flags & VRING_DESC_F_NEXT) == 0)
            break;
        idx = desc[idx].next;
    }

    *chain_len = len;
    return 0;
}

static int userspace_blk_rw(UserspacePciBlkDev *blk, struct virtq_desc *chain,
                            int chain_len, int zone_id,
                            const struct virtio_blk_outhdr *hdr,
                            __u32 *total_len) {
    off_t offset;
    __u32 done = 0;
    bool is_write;
    int i;

    offset = (off_t)(userspace_pci_le64_to_cpu(hdr->sector) *
                     HVISOR_VIRTIO_BLK_SECTOR_SIZE);
    is_write = userspace_pci_le32_to_cpu(hdr->type) == VIRTIO_BLK_T_OUT;

    for (i = 1; i < chain_len - 1; i++) {
        void *buf;
        ssize_t io_len;

        if (offset < 0 ||
            (uint64_t)offset + chain[i].len >
                blk->capacity_sectors * HVISOR_VIRTIO_BLK_SECTOR_SIZE)
            return -EIO;
        if (userspace_pci_translate_addr(zone_id, chain[i].addr, chain[i].len,
                                         &buf) != 0)
            return -EIO;

        if (is_write)
            io_len = pwrite(blk->image_fd, buf, chain[i].len, offset);
        else
            io_len = pread(blk->image_fd, buf, chain[i].len, offset);

        if (io_len < 0)
            return -errno;
        if ((size_t)io_len != chain[i].len)
            return -EIO;
        offset += io_len;
        done += (__u32)io_len;
    }

    *total_len = done;
    return 0;
}

int userspace_pci_blk_init_dev(UserspacePciDev *dev,
                               const struct virtio_pci_config_info *info) {
    (void)info;
    dev->backend.blk.image_fd = -1;
    return userspace_blk_open_image(&dev->backend.blk);
}

void userspace_pci_blk_cleanup_dev(UserspacePciDev *dev) {
    if (dev->backend.blk.image_fd >= 0) {
        close(dev->backend.blk.image_fd);
        dev->backend.blk.image_fd = -1;
    }
}

int userspace_pci_blk_process_vq(UserspacePciDev *dev,
                                 UserspacePciVirtQueue *vq) {
    __u16 used_idx;

    if (dev == NULL || vq == NULL || vq->desc == NULL || vq->avail == NULL ||
        vq->used == NULL)
        return -EINVAL;
    if (vq->queue_size == 0 || dev->backend.blk.image_fd < 0)
        return -EINVAL;

    used_idx = vq->used->idx;
    while (used_idx != vq->avail->idx) {
        struct virtq_desc chain[HVISOR_VIRTIO_BLK_MAX_SEGS + 2];
        struct virtio_blk_outhdr *hdr;
        void *status_buf;
        __u16 slot = used_idx % vq->queue_size;
        __u16 head = vq->avail->ring[slot];
        __u16 used_slot = used_idx % vq->queue_size;
        __u32 total_len = 1;
        uint8_t status = VIRTIO_BLK_S_OK;
        int chain_len;
        int i;
        int ret;

        ret = userspace_blk_collect_chain(vq->desc, head, vq->queue_size, chain,
                                          &chain_len);
        if (ret != 0)
            return ret;
        if (chain_len < 2 || chain[0].len < sizeof(*hdr) ||
            chain[chain_len - 1].len < 1)
            return -EINVAL;
        if ((chain[0].flags & VRING_DESC_F_WRITE) != 0)
            return -EINVAL;
        if ((chain[chain_len - 1].flags & VRING_DESC_F_WRITE) == 0)
            return -EINVAL;

        if (userspace_pci_translate_addr(vq->zone_id, chain[0].addr,
                                         sizeof(*hdr), (void **)&hdr) != 0)
            return -EIO;

        for (i = 1; i < chain_len - 1; i++) {
            bool should_write =
                userspace_pci_le32_to_cpu(hdr->type) != VIRTIO_BLK_T_OUT;

            if (((chain[i].flags & VRING_DESC_F_WRITE) != 0) != should_write)
                return -EINVAL;
        }

        switch (userspace_pci_le32_to_cpu(hdr->type)) {
        case VIRTIO_BLK_T_IN:
        case VIRTIO_BLK_T_OUT:
            ret = userspace_blk_rw(&dev->backend.blk, chain, chain_len,
                                   vq->zone_id, hdr, &total_len);
            break;
        case VIRTIO_BLK_T_GET_ID: {
            void *id_buf;
            size_t copy_len;

            if (chain_len != 3) {
                ret = -EINVAL;
                break;
            }
            if (userspace_pci_translate_addr(vq->zone_id, chain[1].addr,
                                             chain[1].len, &id_buf) != 0) {
                ret = -EIO;
                break;
            }
            copy_len = chain[1].len < sizeof(dev->backend.blk.id)
                           ? chain[1].len
                           : sizeof(dev->backend.blk.id);
            memset(id_buf, 0, chain[1].len);
            memcpy(id_buf, dev->backend.blk.id, copy_len);
            total_len = (__u32)copy_len + 1;
            ret = 0;
            break;
        }
        default:
            ret = -EOPNOTSUPP;
            break;
        }

        if (ret == -EOPNOTSUPP)
            status = VIRTIO_BLK_S_UNSUPP;
        else if (ret != 0)
            status = VIRTIO_BLK_S_IOERR;

        if (userspace_pci_translate_addr(vq->zone_id, chain[chain_len - 1].addr,
                                         1, &status_buf) != 0)
            return -EIO;
        *(uint8_t *)status_buf = status;

        if (ret != 0 && ret != -EOPNOTSUPP)
            total_len = 1;

        vq->used->ring[used_slot].id = head;
        vq->used->ring[used_slot].len = total_len;
        __sync_synchronize();
        used_idx++;
        vq->used->idx = used_idx;
    }

    return 0;
}
