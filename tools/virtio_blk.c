#include "virtio_blk.h"
#include "virtio.h"
#include <stdlib.h>
#include <string.h>
#include <sys/param.h>
#include <errno.h>
#include "log.h"

static void complete_block_operation(BlkDev *dev, struct blkp_req *req, VirtQueue *vq, int err) {
    uint8_t *vstatus = (uint8_t *)(req->iov[req->iovcnt-1].iov_base);
    if (err == EOPNOTSUPP)
        *vstatus = VIRTIO_BLK_S_UNSUPP;
    else if (err != 0) 
        *vstatus = VIRTIO_BLK_S_IOERR;
    else 
        *vstatus = VIRTIO_BLK_S_OK;
    if (err != 0) {
        log_error("virt blk err, num is %d", err);
    }
    update_used_ring(vq, req->idx, 1);
    try_inject_irq(vq, TAILQ_EMPTY(&dev->procq));
	free(req->iov);
    free(req);
}
// get a blk req from procq
static int get_breq(BlkDev *dev, struct blkp_req **req) {
    struct blkp_req *elem;
    elem = TAILQ_FIRST(&dev->procq);
    if (elem == NULL) {
        return 0;
    }
    TAILQ_REMOVE(&dev->procq, elem, link);
    *req = elem;
    return 1;
}

static void blkproc(BlkDev *dev, struct blkp_req *req, VirtQueue *vq) {
    struct iovec *iov = req->iov;
    int n = req->iovcnt, err;
    ssize_t len; 
	
    switch (req->type)
    {
    case VIRTIO_BLK_T_IN:
        len = preadv(dev->img_fd, &iov[1], n - 2, req->offset);
        if (len < 0) {
            log_error("pread failed");
            err = errno;
        }
        break;
    case VIRTIO_BLK_T_OUT:
        len = pwritev(dev->img_fd, &iov[1], n-2, req->offset);
        if (len < 0) {
            log_error("pwrite failed");
            err = errno;
        }
        break;
	case VIRTIO_BLK_T_GET_ID: 
	{
		char s[20] = "hvisor-virblk";
		strncpy(iov[1].iov_base, s, MIN(sizeof(s), iov[1].iov_len));
		break;
	}
    default:
        log_fatal("Operation is not supported");
        err = EOPNOTSUPP;
        break;
    }
    complete_block_operation(dev, req, vq, err);
}

// Every virtio-blk has a blkproc_thread that is used for reading and writing.
static void *blkproc_thread(void *arg)
{
    VirtIODevice *vdev = arg;
    BlkDev *dev = vdev->dev;
    struct blkp_req *breq;
    // get_breq will access the critical section, so lock it.
    pthread_mutex_lock(&dev->mtx);
    
    for (;;) {
        while (get_breq(dev, &breq)) {
            // blk_proc don't access the critical section, so unlock.
            pthread_mutex_unlock(&dev->mtx);
            blkproc(dev, breq, vdev->vqs);
            pthread_mutex_lock(&dev->mtx);
        }

        if (dev->closing)
            break;
        pthread_cond_wait(&dev->cond, &dev->mtx);
    }
    pthread_mutex_unlock(&dev->mtx);
    pthread_exit(NULL);
    return NULL;
}


// create blk dev.
BlkDev *init_blk_dev(VirtIODevice *vdev, uint64_t bsize, int img_fd)
{
    BlkDev *dev = malloc(sizeof(BlkDev));
    dev->config.capacity = bsize;
    dev->config.size_max = bsize;
    dev->config.seg_max = BLK_SEG_MAX;
    dev->img_fd = img_fd;
    dev->closing = 0;
	// TODO: chang to thread poll
    pthread_mutex_init(&dev->mtx, NULL);
    pthread_cond_init(&dev->cond, NULL);
    TAILQ_INIT(&dev->procq);
    pthread_create(&dev->tid, NULL, blkproc_thread, vdev);
    return dev;
}


// handle one descriptor list
static struct blkp_req* virtq_blk_handle_one_request(VirtQueue *vq)
{
    struct blkp_req *breq;
    struct iovec *iov = NULL;
    uint16_t *flags;
    int i, n;
    BlkReqHead *hdr;
    BlkDev *blkDev = vq->dev->dev;
    int err = 0;
    breq = malloc(sizeof(struct blkp_req));
    n = process_descriptor_chain(vq, &breq->idx, &iov, &flags, 0);
	breq->iov = iov;
    if (n < 2 || n > BLK_SEG_MAX + 2) {
        log_error("iov's num is wrong");
        goto err_out;
    }

    if ((flags[0] & VRING_DESC_F_WRITE) != 0) {
        log_error("virt queue's desc chain header should not be writable!");
        goto err_out;
    }

    if (iov[0].iov_len != sizeof(BlkReqHead)) {
        log_error("the size of blk header is %d, it should be %d!", iov[0].iov_len, sizeof(BlkReqHead));
		goto err_out;
    }

    if(iov[n-1].iov_len != 1 || ((flags[n-1] & VRING_DESC_F_WRITE) == 0)) {
        log_error("status iov is invalid!, status len is %d, flag is %d, n is %d", iov[n-1].iov_len, flags[n-1], n);
		goto err_out;
    }

    hdr = (BlkReqHead *) (iov[0].iov_base);
    uint64_t offset = hdr->sector * SECTOR_BSIZE; 
    breq->type = hdr->req_type;
	breq->iovcnt = n;
	breq->offset = offset;

    for (i=1; i<n-1; i++) 
        if (((flags[i] & VRING_DESC_F_WRITE) == 0) != (breq->type == VIRTIO_BLK_T_OUT)) {
            log_error("flag is conflict with operation");
			goto err_out;
        }

    free(flags);
	return breq;

err_out:
	free(flags);
	free(iov);
    free(breq);
	return NULL;
}

int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{
    log_trace("virtio blk notify handler enter");
	BlkDev *blkDev = (BlkDev *)vdev->dev;
	struct blkp_req *breq;
	TAILQ_HEAD(, blkp_req) procq;
	TAILQ_INIT(&procq);
    virtqueue_disable_notify(vq);
    while(!virtqueue_is_empty(vq)) {
        breq = virtq_blk_handle_one_request(vq);
		TAILQ_INSERT_TAIL(&procq, breq, link);
    }
    virtqueue_enable_notify(vq);
	pthread_mutex_lock(&blkDev->mtx);
	TAILQ_CONCAT(&blkDev->procq, &procq, link);
	pthread_cond_signal(&blkDev->cond);
	pthread_mutex_unlock(&blkDev->mtx);
    return 0;
}