#include "virtio_blk.h"
#include "virtio.h"
#include <stdlib.h>
#include <string.h>
#include <sys/param.h>
#include "log.h"
// create blk dev.
BlkDev *init_blk_dev(uint64_t bsize)
{
    BlkDev *dev = malloc(sizeof(BlkDev));
    dev->config.capacity = bsize;
    dev->config.size_max = BLK_SIZE_MAX;
    dev->config.seg_max = BLK_SEG_MAX;
    dev->img_fd = -1;
    return dev;
}

// handle one descriptor list
static void virtq_blk_handle_one_request(VirtQueue *vq)
{
    struct iovec iov[BLK_SEG_MAX+2];
    int i, n, type, writeop;
    uint16_t idx, flags[BLK_SEG_MAX+2];
    BlkReqHead *hdr;
    BlkDev *blkDev = vq->dev->dev;
    ssize_t data_len = 0;
    bool is_support = true, is_err = false;

    n = vq_getchain(vq, &idx, iov, BLK_SEG_MAX+2, flags);

    if (n < 2 || n > BLK_SEG_MAX + 2) {
        log_error("iov's num is wrong");
        return;
    }

    if ((flags[0] & VRING_DESC_F_WRITE) != 0) {
        log_error("virt queue's desc chain header should not be writable!");
        return ;
    }

    if (iov[0].iov_len != sizeof(BlkReqHead)) {
        log_error("the size of blk header is %d, it should be %d!", iov[0].iov_len, sizeof(BlkReqHead));
        return;
    }

    if(iov[n-1].iov_len != 1 || ((flags[n-1] & VRING_DESC_F_WRITE) == 0)) {
        log_error("status iov is invalid!, status len is %d, flag is %d, n is %d", iov[n-1].iov_len, flags[n-1], n);
        return;
    }

    hdr = (BlkReqHead *) (iov[0].iov_base);
    type = hdr->req_type;
    writeop = (type == VIRTIO_BLK_T_OUT);
    uint64_t offset = hdr->sector * SECTOR_BSIZE; 

    for (i=1; i<n-1; i++) {
        if (((flags[i] & VRING_DESC_F_WRITE) == 0) != writeop) {
            log_error("flag is conflict with operation");
            return;
        }
        data_len += iov[i].iov_len;
    }


    switch (type)
    {
        case VIRTIO_BLK_T_IN:
        {
            ssize_t readl = preadv(blkDev->img_fd, &iov[1], n - 2, offset);
            if (readl == -1) {
                log_error("pread failed");
                is_err = true;
            }
            if (readl != data_len) {
                log_error("pread len is wrong");
                is_err = true;
            }
        }
            break;
        case VIRTIO_BLK_T_OUT:
        {
            pwritev(blkDev->img_fd, &iov[1], n - 2, offset);
        }
            break;
        case VIRTIO_BLK_T_GET_ID:
        {
            char s[20] = "hvisor-virblk";
            strncpy(iov[1].iov_base, s, MIN(sizeof(s), iov[1].iov_len));
        }
            break;
        default:
            log_error("unsupported virtqueue request type: %u", hdr->req_type);
            is_support = false;
            break;
    }

    uint8_t *vstatus = (uint8_t *)(iov[n-1].iov_base);
    if (is_err)
        *vstatus = VIRTIO_BLK_S_IOERR;
    else if (is_support)
        *vstatus = VIRTIO_BLK_S_OK;
    else
        *vstatus = VIRTIO_BLK_S_UNSUPP;
    update_used_ring(vq, idx, 1);
}

int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{
    log_trace("virtio blk notify handler enter");
    /*
    1. 从可用环中取出请求,
    2. 将请求池的各个请求映射为文件进行处理
    */
    virtqueue_disable_notify(vq);
    while(!virtqueue_is_empty(vq)) {
        // uint16_t desc_idx = virtqueue_pop_desc_chain_head(vq); //描述符链头
        // log_debug("avail_idx is %d, last_avail_idx is %d, desc_head_idx is %d", vq->avail_ring->idx, vq->last_avail_idx, desc_idx);
        virtq_blk_handle_one_request(vq);
    }
    virtqueue_enable_notify(vq);
    return 0;
}