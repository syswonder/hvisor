#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "virtio.h"
#include "hvisor.h"
#include "virtio_blk.h"
#include "virtio_net.h"
#include "log.h"
#include <sys/mman.h>
#include <sys/uio.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <sys/ioctl.h>

VirtIODevice *vdevs[16];
int vdevs_num;
// 拥有整个non root mem区域的virt_addr
void *virt_addr;
void *phys_addr;
int img_fd;
#define NON_ROOT_PHYS_START 0x70000000
#define NON_ROOT_PHYS_SIZE 0x8000000
#define NON_ROOT_PHYS_START2 0x81000000
#define NON_ROOT_PHYS_SIZE2 0x30000000
// TODO: 根据配置文件初始化device. 可以参考rhyper的emu_virtio_mmio_init函数; 也可以命令行随时指定
int init_virtio_devices()
{
    img_fd = open("ubuntu-20.04-rootfs_ext4.img", O_RDWR);
    // img_fd = open("virtio_ext4.img", O_RDWR);

    if (img_fd == -1) {
        log_error("cannot open virtio_blk.img, Error code is %d", errno);
        return -1;
    }

    int mem_fd = open("/dev/mem", O_RDWR | O_SYNC);
    phys_addr = NON_ROOT_PHYS_START2;
    virt_addr = mmap(NULL, NON_ROOT_PHYS_SIZE2, PROT_READ | PROT_WRITE, MAP_SHARED, mem_fd, phys_addr);
    log_info("mmap virt addr is %#x", virt_addr);
    create_virtio_device(VirtioTBlock, 1);
    create_virtio_device(VirtioTNet, 1);
    return 0;
}

// create a virtio device.
VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t cell_id)
{
    VirtIODevice *vdev = NULL;
    switch (dev_type)
    {
    case VirtioTBlock:
        vdev = calloc(1, sizeof(VirtIODevice));
        init_mmio_regs(&vdev->regs, dev_type);
        vdev->base_addr = 0xa003e00;
        vdev->len = 0x200;
        vdev->cell_id = cell_id;
        vdev->irq_id = 67;
        vdev->type = dev_type;
        vdev->regs.dev_feature = VIRTIO_BLK_F_SEG_MAX | VIRTIO_BLK_F_SIZE_MAX | VIRTIO_F_VERSION_1;
        vdev->dev = init_blk_dev(BLK_SIZE_MAX); // 256MB
        init_virtio_queue(vdev, dev_type);
        break;
    case VirtioTNet:
        vdev = calloc(1, sizeof(VirtIODevice));
        init_mmio_regs(&vdev->regs, dev_type);
        vdev->base_addr = 0xa003c00;
        vdev->len = 0x200;
        vdev->cell_id = cell_id;
        vdev->irq_id = 68;
        // TODO: 先按着acrn的来吧, 之后再试着弄CSUM之类的
        vdev->regs.dev_feature = VIRTIO_NET_FEATURES;
        vdev->type = dev_type;
        uint8_t mac[] = {0x00, 0x16, 0x3E, 0x10, 0x10, 0x10};
        vdev->dev = init_net_dev(mac);
        init_virtio_queue(vdev, dev_type);
        virtio_net_init(vdev, "tap0");
    default:
        break;
    }
    vdevs[vdevs_num++] = vdev;
    return vdev;
}

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type)
{
    VirtQueue *vq = NULL;
    switch (type)
    {
    case VirtioTBlock:
        vdev->vqs_len = 1;
        vq = malloc(sizeof(VirtQueue));
        virtqueue_reset(vq, 0);
        vq->queue_num_max = VIRTQUEUE_BLK_MAX_SIZE;
        vq->notify_handler = virtio_blk_notify_handler;
        vdev->vqs = vq;
        break;
    case VirtioTNet:
        vdev->vqs_len = VIRTIO_NET_MAXQ;
        vq = malloc(sizeof(VirtQueue) * VIRTIO_NET_MAXQ);
        for (int i = 0; i < VIRTIO_NET_MAXQ; ++i) {
            virtqueue_reset(vq, i);
            vq[i].queue_num_max = VIRTQUEUE_NET_MAX_SIZE;
        }
        vq[VIRTIO_NET_RXQ].notify_handler = virtio_net_rxq_notify_handler;
        vq[VIRTIO_NET_TXQ].notify_handler = virtio_net_txq_notify_handler;
        vdev->vqs = vq;
    default:
        break;
    }
}

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type)
{
    regs->device_id = type;
    regs->queue_sel = 0;
}

void virtio_dev_reset(VirtIODevice *vdev)
{
    log_trace("virtio dev reset");
    vdev->regs.status = 0;
    vdev->regs.interrupt_status = 0;
    int idx = vdev->regs.queue_sel;
    vdev->vqs[idx].ready = 0;
    for(uint32_t i=0; i<vdev->vqs_len; i++) {
        virtqueue_reset(&vdev->vqs[i], i);
    }
    vdev->activated = false;
}

void virtqueue_reset(VirtQueue *vq, int idx)
{
    vq += idx;
    // don't reset notify handler
    void *addr = vq->notify_handler;
    memset(vq, 0, sizeof(VirtQueue));
    vq->vq_idx = idx;
    vq->notify_handler = addr;
}

// check if virtqueue has new requests
bool virtqueue_is_empty(VirtQueue *vq)
{
    if(vq->avail_ring == NULL) {
        log_error("virtqueue's avail ring is invalid");
        return true;
    }
    if (vq->last_avail_idx == vq->avail_ring->idx)
        return true;
    else
        return false;
}

// get the first descriptor chain's head idx in descriptor table.
uint16_t virtqueue_pop_desc_chain_head(VirtQueue *vq)
{
    uint16_t ring_idx = vq->last_avail_idx % vq->num;
    vq->last_avail_idx++;
    return vq->avail_ring->ring[ring_idx];
}

bool desc_is_writable(VirtqDesc *desc_table, uint16_t idx)
{
    if (desc_table[idx].flags & VRING_DESC_F_WRITE)
        return true;
    return false;
}

void* get_virt_addr(void *addr)
{
    return virt_addr - phys_addr + addr;
}

// get non root linux's ipa
void* get_phys_addr(void *addr)
{
    return addr - virt_addr + phys_addr;
}

void virtqueue_disable_notify(VirtQueue *vq) {
    vq->used_ring->flags |= (uint16_t)VRING_USED_F_NO_NOTIFY;
}

void virtqueue_enable_notify(VirtQueue *vq) {
    vq->used_ring->flags &= !(uint16_t)VRING_USED_F_NO_NOTIFY;
}

void virtqueue_set_desc_table(VirtQueue *vq)
{
    log_trace("desc table ipa is %#x", vq->desc_table_addr);
    vq->desc_table = (VirtqDesc *)(virt_addr + vq->desc_table_addr - phys_addr);
}

void virtqueue_set_avail(VirtQueue *vq)
{
    log_trace("avail ring ipa is %#x", vq->avail_addr);
    vq->avail_ring = (VirtqAvail *)(virt_addr + vq->avail_addr - phys_addr);
}

void virtqueue_set_used(VirtQueue *vq)
{
    log_trace("used ring ipa is %#x", vq->used_addr);
    vq->used_ring = (VirtqUsed *)(virt_addr + vq->used_addr - phys_addr);
}

/*
 * Helper inline for vq_getchain(): record the i'th "real"
 * descriptor.
 * Return 0 on success and -1 when i is out of range  or mapping
 *        fails.
 */
static inline int
_vq_record(int i, volatile VirtqDesc *vd,
           struct iovec *iov, int n_iov, uint16_t *flags) {
    // 将vd指向的描述符记录在iov中的第i个元素中
    void *host_addr;

    if (i >= n_iov)
        return -1;
    host_addr = get_virt_addr(vd->addr);
    iov[i].iov_base = host_addr;
    iov[i].iov_len = vd->len;
    if (flags != NULL)
        flags[i] = vd->flags;
    return 0;
}
/// record one descriptor list to iov
/// \param pidx the first descriptor's idx in descriptor list.
/// \param n_iov the max num of iov
/// \param flags each descriptor's flags
/// \return the valid num of iov
int vq_getchain(VirtQueue *vq, uint16_t *pidx,
                struct iovec *iov, int n_iov, uint16_t *flags)
{
    uint16_t next, idx;
    volatile VirtqDesc *vdesc;
    idx = vq->last_avail_idx;
    vq->last_avail_idx++;
    *pidx = next = vq->avail_ring->ring[idx & (vq->num - 1)];

    for (int i=0; i < vq->num; next = vdesc->next) {
        vdesc = &vq->desc_table[next];
        if (_vq_record(i, vdesc, iov, n_iov, flags)) {
            log_error("vq record failed");
            return -1;
        }
        i++;
        if ((vdesc->flags & VRING_DESC_F_NEXT) == 0)
            return i;
    }
    log_error("desc not end?");
    return -1;
}

void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen)
{
    volatile VirtqUsed *used_ring;
    volatile VirtqUsedElem *elem;
    uint16_t used_idx, mask;
    used_ring = vq->used_ring;
    used_idx = used_ring->idx;
    mask = vq->num - 1;
    elem = used_ring[used_idx++ & mask];
    elem->id = idx;
    elem->len = iolen;
    used_ring->idx = used_idx;
}
static uint64_t virtio_mmio_read(VirtIODevice *vdev, uint64_t offset, unsigned size)
{
    log_trace("virtio mmio read at %#x", offset);
    if (!vdev) {
        /* If no backend is present, we treat most registers as
         * read-as-zero, except for the magic number, version and
         * vendor ID. This is not strictly sanctioned by the virtio
         * spec, but it allows us to provide transports with no backend
         * plugged in which don't confuse Linux's virtio code: the
         * probe won't complain about the bad magic number, but the
         * device ID of zero means no backend will claim it.
         */
        switch (offset) {
        case VIRTIO_MMIO_MAGIC_VALUE:
            return VIRT_MAGIC;
        case VIRTIO_MMIO_VERSION:
            return VIRT_VERSION;
        case VIRTIO_MMIO_VENDOR_ID:
            return VIRT_VENDOR;
        default:
            return 0;
        }
    }

    if (offset >= VIRTIO_MMIO_CONFIG) {
        offset -= VIRTIO_MMIO_CONFIG;
        // the first member of vdev->dev must be config.
        return *(uint64_t *)(vdev->dev + offset);
    }

    if (size != 4) {
        log_error("virtio-mmio-read: wrong size access to register!");
        return 0;
    }

    switch (offset) {
    case VIRTIO_MMIO_MAGIC_VALUE:
        return VIRT_MAGIC;
    case VIRTIO_MMIO_VERSION:
        return VIRT_VERSION;
    case VIRTIO_MMIO_DEVICE_ID:
        return vdev->regs.device_id;
    case VIRTIO_MMIO_VENDOR_ID:
        return VIRT_VENDOR;
    case VIRTIO_MMIO_DEVICE_FEATURES:
        if (vdev->regs.dev_feature_sel) {
            return vdev->regs.dev_feature >> 32;
        } else {
            return vdev->regs.dev_feature;
        }
    case VIRTIO_MMIO_QUEUE_NUM_MAX:
        return vdev->vqs[vdev->regs.queue_sel].queue_num_max;
    case VIRTIO_MMIO_QUEUE_READY:
        return vdev->vqs[vdev->regs.queue_sel].ready;
    case VIRTIO_MMIO_INTERRUPT_STATUS:
        return vdev->regs.interrupt_status;
    case VIRTIO_MMIO_STATUS:
        return vdev->regs.status;
    case VIRTIO_MMIO_CONFIG_GENERATION:
        return vdev->regs.generation;
   case VIRTIO_MMIO_SHM_LEN_LOW:
   case VIRTIO_MMIO_SHM_LEN_HIGH:
        /*
         * VIRTIO_MMIO_SHM_SEL is unimplemented
         * according to the linux driver, if region length is -1
         * the shared memory doesn't exist
         */
        return -1;
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

static void virtio_mmio_write(VirtIODevice *vdev, uint64_t offset, uint64_t value, unsigned size)
{
    log_trace("virtio mmio write at %#x, value is %d\n", offset, value);
    VirtMmioRegs *regs = &vdev->regs;
    VirtQueue *vqs = vdev->vqs;
    if (!vdev) {
        /* If no backend is present, we just make all registers
         * write-ignored. This allows us to provide transports with
         * no backend plugged in.
         */
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
        if (value) {
            regs->dev_feature_sel = 1;
        } else {
            regs->dev_feature_sel = 0;
        }
        break;
    case VIRTIO_MMIO_DRIVER_FEATURES:
        if (regs->drv_feature_sel) {
            regs->drv_feature |= value << 32;
        } else {
            regs->drv_feature |= value;
        }
        break;
    case VIRTIO_MMIO_DRIVER_FEATURES_SEL:
        if (value) {
            regs->drv_feature_sel = 1;
        } else {
            regs->drv_feature_sel = 0;
        }
        break;
    case VIRTIO_MMIO_QUEUE_SEL:
        if (value < vdev->vqs_len) {
            regs->queue_sel = value;
        }
        break;
    case VIRTIO_MMIO_QUEUE_NUM:
        vqs[regs->queue_sel].num = value;
        log_trace("virtqueue num is %d", value);
        break;
    case VIRTIO_MMIO_QUEUE_READY:
        vqs[regs->queue_sel].ready = value;
        break;
    case VIRTIO_MMIO_QUEUE_NOTIFY:
        log_debug("queue notify begin");
        regs->interrupt_status = VIRTIO_MMIO_INT_VRING;
        if (value < vdev->vqs_len) {
            log_trace("queue notify ready, handler addr is %#x", vqs[value].notify_handler);
            vqs[value].notify_handler(vdev, &vqs[value]);
        }
        log_debug("queue notify end");
        break;
    case VIRTIO_MMIO_INTERRUPT_ACK:
        regs->interrupt_status &= !value;
        regs->interrupt_ack = value;
        break;
    case VIRTIO_MMIO_STATUS:
        regs->status = value;
        if (regs->status == 0) {
            virtio_dev_reset(vdev);
        }
        break;
    case VIRTIO_MMIO_QUEUE_DESC_LOW:
        vqs[regs->queue_sel].desc_table_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_DESC_HIGH:
        vqs[regs->queue_sel].desc_table_addr |= value << 32;
        virtqueue_set_desc_table(&vqs[regs->queue_sel]);
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_LOW:
        vqs[regs->queue_sel].avail_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_HIGH:
        vqs[regs->queue_sel].avail_addr |= value << 32;
        virtqueue_set_avail(&vqs[regs->queue_sel]);
        break;
    case VIRTIO_MMIO_QUEUE_USED_LOW:
        vqs[regs->queue_sel].used_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_USED_HIGH:
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

static inline bool in_range(uint64_t value, uint64_t lower, uint64_t len)
{
    return ((value >= lower) && (value < (lower + len)));
}

void virtio_finish_req(uint64_t tar_cpu, uint64_t value, uint8_t is_cfg)
{
    // TODO: 多线程时要加锁.
    struct device_res *res;
    unsigned int res_idx = device_region->res_idx;
    while (res_idx - device_region->last_res_idx == MAX_REQ);
    res = &device_region->res_list[res_idx & (MAX_REQ - 1)];
    res->value = value;
    res->tar_cpu = tar_cpu;
    res->is_cfg = is_cfg;
    // TODO: Barrier
    device_region->res_idx++;
    ioctl(ko_fd, HVISOR_FINISH);
}

int virtio_handle_req(struct device_req *req)
{
    int i;
    uint64_t value;
    for (i = 0; i < vdevs_num; ++i) {
        if ((req->src_cell == vdevs[i]->cell_id) && in_range(req->address, vdevs[i]->base_addr, vdevs[i]->len))
            break;
    }
    if (i == vdevs_num) {
        log_error("no matched virtio dev");
        return -1;
    }
    VirtIODevice *vdev = vdevs[i];
    uint64_t offs = req->address - vdev->base_addr;
    if (req->is_write) {
        virtio_mmio_write(vdev, offs, req->value, req->size);
    } else {
        value = virtio_mmio_read(vdev, offs, req->size);
        log_debug("read value is %d\n", value);
    }
    if (req->is_cfg) {
        // If a request is a control not a data request
        virtio_finish_req(req->src_cpu, value, 1);
    }
    log_debug("src_cell is %d, src_cpu is %lld", req->src_cell, req->src_cpu);
    return 0;
}