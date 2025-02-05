#define _GNU_SOURCE
#include "virtio.h"
#include "virtio_console.h"
#include<stdlib.h>
#include<fcntl.h>
#include "log.h"
#include <errno.h>
#include <termios.h>
static uint8_t trashbuf[1024];

ConsoleDev *init_console_dev() {
    ConsoleDev *dev = (ConsoleDev *)malloc(sizeof(ConsoleDev));
    dev->config.cols = 80;
    dev->config.rows = 25;
    dev->master_fd = -1;
    dev->rx_ready = -1;
    dev->event = NULL;
    return dev;
}

#ifdef LOONGARCH64
double last_process_timestamp = -1;
const int LOONGARCH64_CONSOLE_PROCESS_INTERVAL_MS = 500;
#endif

static void virtio_console_event_handler(int fd, int epoll_type, void *param) {
    log_debug("%s", __func__);
    VirtIODevice *vdev = (VirtIODevice *)param;
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;
    VirtQueue *vq = &vdev->vqs[CONSOLE_QUEUE_RX];
    int n;
    ssize_t len;
    struct iovec *iov = NULL;
    uint16_t idx;
    if (epoll_type != EPOLLIN || fd != dev->master_fd) {
        log_error("Invalid console event");
        return ;
    }
    if (dev->master_fd <= 0 || vdev->type != VirtioTConsole) {
        log_error("console event handler should not be called");
        return ;
    } 

    // log_warn("wheatfox: (%s) in console event handler, fd=%d, epoll_type=%d, vdev->type=%d", __func__, fd, epoll_type, vdev->type);

#ifdef LOONGARCH64
    // TODO: gettimeofday on loongarch linux on 3A5000 board return error values! thus 
    // the below code is commented out for now in order to use screen to connect nonroot bash successfully - wheatfox 2025.2.5

    // we wait until current timestamp - last_process_timestamp > LOONGARCH64_CONSOLE_PROCESS_INTERVAL_MS
    // to avoid too frequent ipi sending that cause ipi failure on loongson 3A5000 board
    // struct timeval tv;
    // double current_timestamp;
    // log_warn("wheatfox: (%s) start blocking...", __func__);
    // while(1) {
    //     if (last_process_timestamp < 0) {
    //         break;
    //     }
    //     gettimeofday(&tv, NULL);
    //     current_timestamp = (tv.tv_sec) * 1000 + (tv.tv_usec) / 1000 ; // convert tv_sec & tv_usec to millisecond
    //     if (current_timestamp - last_process_timestamp > LOONGARCH64_CONSOLE_PROCESS_INTERVAL_MS) {
    //         break;
    //     }
    // }
    // log_warn("wheatfox: (%s) while 1 done, current_timestamp is %f, last_process_timestamp is %f", __func__, current_timestamp, last_process_timestamp);
    // last_process_timestamp = current_timestamp;
#endif

    if (dev->rx_ready <= 0) {
        read(dev->master_fd, trashbuf, sizeof(trashbuf));
        // log_error("console rxq is not ready, your input data which is %s will be discarded", trashbuf);
        return ;
    }
    if (virtqueue_is_empty(vq)) {
        read(dev->master_fd, trashbuf, sizeof(trashbuf));
        virtio_inject_irq(vq);
        // log_error("virtqueue is empty, your input data which is %s will be discarded", trashbuf);
        return ;
    }
    
    while (!virtqueue_is_empty(vq)) {
        n = process_descriptor_chain(vq, &idx, &iov, NULL, 0);
        if (n < 1) {
            log_error("process_descriptor_chain failed");
            break;
        }
        len = readv(dev->master_fd, iov, n);
        // log_info("wheatfox: (%s) readv done, len is %d, vq->last_avail_idx is %d", __func__, len, vq->last_avail_idx);
        // if len is not 0, print the data
        if (len > 0) {
            // log_info("wheatfox: (%s) readv done, len is %d, data is at iov@%#x, dump:\nRAW:[", __func__, len, iov);
            // now starting from iov base, print exactly len bytes
            for (int i = 0; i < len; i++) {
                log_printf("%c", *(char*)&iov->iov_base[i]);
            }
            log_printf("] vq->last_avail_idx is %d\n", vq->last_avail_idx);
        }
        if (len < 0 && errno == EWOULDBLOCK) {
            log_debug("no more bytes");
			vq->last_avail_idx--;
            // log_error("wheatfox: (%s) no more bytes, vq->last_avail_idx --> %d", __func__, vq->last_avail_idx);
            free(iov);
			break;
        } else if (len < 0) {
            log_trace("Failed to read from console, errno is %d", errno);
			vq->last_avail_idx--;
            // log_error("wheatfox: (%s) Failed to read from console, errno is %d[%s], vq->last_avail_idx --> %d", __func__, errno, strerror(errno), vq->last_avail_idx);
            free(iov);
            break;
        } 
        update_used_ring(vq, idx, len);
        // log_info("wheatfox: (%s) update_used_ring done", __func__);
        free(iov);
    }
    virtio_inject_irq(vq);
    // log_trace("wheatfox: (%s) ok, finish handling console event", __func__);
    return ;
}

int virtio_console_init(VirtIODevice *vdev) {
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;
    int master_fd, slave_fd;
    char *slave_name;
    struct termios term_io;

    master_fd = posix_openpt(O_RDWR | O_NOCTTY);
    if (master_fd < 0) {
        log_error("Failed to open master pty, errno is %d", errno);
    }
    if (grantpt(master_fd) < 0) {
        log_error("Failed to grant pty, errno is %d", errno);
    }
    if (unlockpt(master_fd) < 0) {
        log_error("Failed to unlock pty, errno is %d", errno);
    }
    dev->master_fd = master_fd;

    slave_name = ptsname(master_fd);
    if (slave_name == NULL) {
        log_error("Failed to get slave name, errno is %d", errno);
    }
    log_warn("char device redirected to %s, fd=%d", slave_name, master_fd);
    // Disable line discipline to prevent the TTY 
    // from echoing the characters sent from the master back to the master.
    slave_fd = open(slave_name, O_RDWR);
    tcgetattr(slave_fd, &term_io);
    cfmakeraw(&term_io);
    tcsetattr(slave_fd, TCSAFLUSH, &term_io); 
    close(slave_fd);

    if (set_nonblocking(dev->master_fd) < 0) {
        dev->master_fd = -1;
        close(dev->master_fd);
        log_error("Failed to set nonblocking mode, fd closed!");
    }

    dev->event = add_event(dev->master_fd, EPOLLIN, virtio_console_event_handler, vdev);

    if (dev->event == NULL) {
        log_error("Can't register console event");
        close(master_fd);
        dev->master_fd = -1;
        return -1;
    }

    vdev->virtio_close = virtio_console_close;
    return 0;
}

int virtio_console_rxq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("%s", __func__);
    
    // log_info("wheatfox: (%s) start, vq@%#x", __func__, vq);
    
    ConsoleDev *dev = (ConsoleDev *)vdev->dev;

    // log_info("wheatfox: (%s) dev@%#x, dev->rx_ready is %d", __func__, dev, dev->rx_ready);

    if (dev->rx_ready <= 0) {
        dev->rx_ready = 1;
        // log_info("wheatfox: (%s) calling virtqueue_disable_notify", __func__);
        virtqueue_disable_notify(vq);
        // log_info("wheatfox: (%s) virtqueue_disable_notify done", __func__);
    }
    // log_info("wheatfox: (%s) end, vq@%#x", __func__, vq);
    return 0;
}

static void virtq_tx_handle_one_request(ConsoleDev *dev, VirtQueue *vq) {
    int n;
    uint16_t idx;
    ssize_t len;
    struct iovec *iov = NULL;
    static int count = 0;
    count++;
    if (dev->master_fd <= 0) {
        log_error("Console master fd is not ready");
        return ;
    }

    n = process_descriptor_chain(vq, &idx, &iov, NULL, 0);
    // if (count % 100 == 0) {
    //     log_info("console txq: n is %d, data is ", n);
    //     for (int i=0; i<iov->iov_len; i++)
    //         log_printf("%c", *(char*)&iov->iov_base[i]);
    //     log_printf("\n");
    // }

    log_printf("wheatfox: (%s) console txq: n is %d, iov at ", __func__, n);
    for (int i = 0; i < n; i++) {
        log_printf("[%d:%#x|%d] ", i, iov[i].iov_base, iov[i].iov_len);
    }
    log_printf("\n");
#if 1
    for (int i = 0; i < n; i++) {
        log_printf("RAW:[");
        for (int j = 0; j < iov[i].iov_len; j++) {
            char x = *(char*)&iov[i].iov_base[j];
            // if x == \t or \n or \r, replace it with space
            if (x == '\t' || x == '\n' || x == '\r') {
                x = ' ';
            }
            log_printf("%c", x);
        }
        log_printf("]\n");
    }
#endif

    if (n < 1) {
        return ;
    }

    len = writev(dev->master_fd, iov, n);
    if (len < 0) {
        log_error("Failed to write to console, errno is %d", errno);
    }
    update_used_ring(vq, idx, 0);
    free(iov);
}

int virtio_console_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    log_debug("%s", __func__);
    // log_info("wheatfox: (%s) start, vq@%#x", __func__, vq);
    while (!virtqueue_is_empty(vq)) {
        virtqueue_disable_notify(vq);
        while(!virtqueue_is_empty(vq)) {
            virtq_tx_handle_one_request(vdev->dev, vq);
        }
        virtqueue_enable_notify(vq);
    }    
    virtio_inject_irq(vq);
    // log_info("wheatfox: (%s) end, vq@%#x", __func__, vq);
    return 0;
}

void virtio_console_close(VirtIODevice *vdev) {
    ConsoleDev *dev = vdev->dev;
    close(dev->master_fd);
    free(dev->event);
    free(dev);
    free(vdev->vqs);
    free(vdev);
}