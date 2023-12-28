#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "virtio.h"
#include "hvisor.h"
#include "virtio_blk.h"
#include "log.h"
#include <sys/mman.h>
#include <fcntl.h>

VirtIODevice *dev_blk;
// 拥有整个non root mem区域的virt_addr
uint64_t *virt_addr;

// TODO: 根据配置文件初始化device. 可以参考rhyper的emu_virtio_mmio_init函数
int init_virtio_devices() 
{
    int mem_fd = open("/dev/mem", O_RDWR | O_NDELAY);
    virt_addr = mmap(NULL, 0x8000000, PROT_READ | PROT_WRITE, MAP_SHARED, mem_fd, 0x70000000);
    log_info("mmap virt addr is %#x", virt_addr);
    dev_blk = create_virtio_device(VirtioTBlock);
    return 0;
}

// create a virtio device.
VirtIODevice *create_virtio_device(VirtioDeviceType dev_type) 
{
    VirtIODevice *vdev = NULL;
    switch (dev_type)
    {
    case VirtioTBlock:
        vdev = calloc(1, sizeof(VirtIODevice));
        init_mmio_regs(&vdev->regs, dev_type);
        vdev->base_addr = 0xa003e00;
        vdev->dev.features = VIRTIO_BLK_F_SEG_MAX | VIRTIO_BLK_F_SIZE_MAX | VIRTIO_F_VERSION_1;
        vdev->dev.irq_id = 67;
        vdev->dev.type = dev_type;
        vdev->dev.config = init_blk_config(524288); // 256MB
        init_virtio_queue(vdev, dev_type);
        break;
    
    default:
        break;
    }
    return vdev;
}

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type) 
{   
    VirtQueue *vq = NULL;
    switch (type)
    {
    case VirtioTBlock:
        vdev->regs.queue_num_max = VIRTQUEUE_BLK_MAX_SIZE;
        vdev->vqs_len = 1;
        vq = malloc(sizeof(VirtQueue));
        virtqueue_reset(vq, 0);
        vq->notify_handler = virtio_blk_notify_handler;
        log_debug("notify handler addr is %#x", vq->notify_handler);
        vdev->vqs = vq;
        break;
    default:
        break;
    }
}

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type) 
{
    regs->device_id = type;
    regs->queue_sel = 0;
}

// create blk config.
BlkConfig *init_blk_config(uint64_t bsize) 
{
    BlkConfig *config = malloc(sizeof(BlkConfig));
    config->capacity = bsize;
    config->size_max = BLK_SIZE_MAX;
    config->seg_max = BLK_SEG_MAX;
    return config;
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
    vdev->dev.activated = false;
}

void virtqueue_reset(VirtQueue *vq, int idx) 
{
    void *addr = vq->notify_handler;
    memset(vq, 0, sizeof(VirtQueue));
    vq->vq_idx = idx;
    vq->notify_handler = addr;
}   

int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{
    log_debug("virtio blk notify handler enter");
    return 0;
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
        // TODO: 不知道需不需要判断size
        return *(uint64_t *)(vdev->dev.config + offset);
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
            return vdev->dev.features >> 32;
        } else {
            return vdev->dev.features;
        }
    case VIRTIO_MMIO_QUEUE_NUM_MAX:
        return vdev->regs.queue_num_max;
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
    VirtDev *dev = &vdev->dev;
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
            vdev->driver_features |= value << 32;
        } else {
            vdev->driver_features |= value;
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
        break;
    case VIRTIO_MMIO_QUEUE_READY:
        vqs[regs->queue_sel].ready = value;
        break;
    case VIRTIO_MMIO_QUEUE_NOTIFY:
        log_debug("queue notify begin");
        regs->interrupt_status = VIRTIO_MMIO_INT_VRING;
        if (value < vdev->vqs_len) {
            log_debug("queue notify ready, handler addr is %#x", vqs[value].notify_handler);
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
        // TODO: 设置下desc table的地址???
        // virtqueue_set_desc_table();
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_LOW:
        vqs[regs->queue_sel].avail_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_AVAIL_HIGH:
        // TODO: 设置avail ring的地址
        vqs[regs->queue_sel].avail_addr |= value << 32;
        break;
    case VIRTIO_MMIO_QUEUE_USED_LOW:
        vqs[regs->queue_sel].used_addr |= value & UINT32_MAX;
        break;
    case VIRTIO_MMIO_QUEUE_USED_HIGH:
        // TODO: 设置used ring的地址
        vqs[regs->queue_sel].used_addr |= value << 32;
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

int virtio_handle_req(struct device_req *req, struct device_result *res) 
{
    uint64_t offs = req->address - dev_blk->base_addr;
    if (req->is_write) {
        virtio_mmio_write(dev_blk, offs, req->value, req->size);
    } else {
        res->value = virtio_mmio_read(dev_blk, offs, req->size);
        log_debug("read value is %d\n", res->value);
    }
    res->src_cpu = req->src_cpu;
    res->is_cfg = req->is_cfg;
    if (!res->is_cfg) {
        res->value = dev_blk->dev.irq_id;
    }
    log_debug("src_cell is %d, src_cpu is %lld", req->src_cell, req->src_cpu);
    return 0;
}