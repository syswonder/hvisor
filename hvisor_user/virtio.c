#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "virtio.h"
#include "hvisor.h"
#include "virtio_blk.h"
#include "log.h"
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
VirtIODevice *dev_blk;
// 拥有整个non root mem区域的virt_addr
void *virt_addr;
void *phys_addr;
int img_fd;
// TODO: 根据配置文件初始化device. 可以参考rhyper的emu_virtio_mmio_init函数
int init_virtio_devices() 
{
    // img_fd = open("ubuntu-20.04-rootfs_ext4.img", O_RDWR);
    img_fd = open("virtio_ext4.img", O_RDWR);

    if (img_fd == -1) {
        log_error("cannot open virtio_blk.img, Error code is %d", errno);
        return -1;
    }

    int mem_fd = open("/dev/mem", O_RDWR | O_NDELAY);
    phys_addr = 0x70000000;
    virt_addr = mmap(NULL, 0x8000000, PROT_READ | PROT_WRITE, MAP_SHARED, mem_fd, phys_addr);
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
        vdev->dev.config = init_blk_config(BLK_SIZE_MAX); // 256MB
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

// check if virtqueue has new requests
bool virtqueue_is_empty(VirtQueue *vq)
{
    if(vq->avail_ring == NULL) {
        log_error("virtqueue's avail ring is invalid");
        return true;
    }
    uint16_t last_avail_idx = vq->last_avail_idx;
    uint16_t idx = vq->avail_ring->idx;
    if (last_avail_idx == idx) 
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

// 获取non root linux的ipa
void* get_phys_addr(void *addr)
{
    return addr - virt_addr + phys_addr;
}
// 解决一个描述符链
void virtqueue_handle_request(VirtQueue *vq, uint16_t desc_head_idx) 
{
    VirtqDesc *desc_table = vq->desc_table;
    uint16_t desc_idx = desc_head_idx;
    // handle head
    if(desc_is_writable(desc_table, desc_idx)) {
        log_error("virt queue's desc chain header should not be writable!");
        return ;
    }
    log_debug("desc_table addr is %#x, idx is %d, blkreqhead ipa is %#x", get_phys_addr(desc_table), desc_idx, desc_table[desc_idx].addr);
    BlkReqHead *head = (BlkReqHead *)get_virt_addr(desc_table[desc_idx].addr);
    log_debug("head addr is %#x", head);
    desc_idx = desc_table[desc_idx].next;
    // 获取本次请求的数据总长度
    uint32_t req_len = 0; 
    bool is_support = true;
    char *buf = NULL;
    switch (head->req_type)
    {
    case VIRTIO_BLK_T_IN:
    case VIRTIO_BLK_T_OUT:
    {
        uint64_t offset = head->sector * SECTOR_BSIZE; // 这个是对的, 512一个扇区大小
        while (desc_table[desc_idx].flags & VRING_DESC_F_NEXT)  
        {
            log_debug("desc_idx is %d, addr is %#x, len is %d", desc_idx, desc_table[desc_idx].addr, desc_table[desc_idx].len);
            buf = get_virt_addr(desc_table[desc_idx].addr);
            if (head->req_type == VIRTIO_BLK_T_IN){
                log_debug("read offset is %d", offset);
                ssize_t readl = pread(img_fd, buf, desc_table[desc_idx].len, offset);
                if (readl == -1) {
                    log_error("pread failed");
                }
                if (readl != desc_table[desc_idx].len) {
                    log_error("pread len is wrong");
                }
                // printf("pread buf is ");
                // for (int i=0; i<desc_table[desc_idx].len; i++) 
                //     printf("%c", buf[i]);
                // printf("\n");


                // char *pbuf = (char *)malloc(desc_table[desc_idx].len*2 + 20);
                // int poff = 0;
                // poff += sprintf(pbuf, "pread buf is ");
                // for (int i=0; i<desc_table[desc_idx].len; i++) 
                //     poff += sprintf(pbuf + poff, "%x",buf[i]);
                // sprintf(pbuf + poff, "\n");
                // log_debug("%s", pbuf);
                // free(pbuf);
            }
            else {
                log_debug("write offset is %d", offset);
                pwrite(img_fd, buf, desc_table[desc_idx].len, offset);
            } 
            offset += desc_table[desc_idx].len;
            req_len += desc_table[desc_idx].len;
            desc_idx = desc_table[desc_idx].next;
        }
    }
        break;
    case VIRTIO_BLK_T_GET_ID:
    {
        log_debug("virtio get id");
        char s[20] = "virtio-lgw-blk";
        buf = get_virt_addr(desc_table[desc_idx].addr);
        memcpy(buf, s, 20);
        req_len = desc_table[desc_idx].len;
        desc_idx = desc_table[desc_idx].next;
    }
        break;
    default:
        log_error("unsupported virtqueue request type: %u", head->req_type);
        is_support = false;
        while (desc_table[desc_idx].flags & VRING_DESC_F_NEXT) {
            desc_idx = desc_table[desc_idx].next;
        }
        break;
    }

    // the status field of desc chain
    if (!desc_is_writable(desc_table, desc_idx)) {
        log_error("Failed to write virt blk queue desc status");
        return ;
    }
    uint8_t *vstatus = (uint8_t *)get_virt_addr(desc_table[desc_idx].addr);
    if (is_support) 
        *vstatus = VIRTIO_BLK_S_OK;
    else 
        *vstatus = VIRTIO_BLK_S_UNSUPP;
    // update used ring
    VirtqUsed *used_ring = vq->used_ring;
    uint16_t used_idx = used_ring->idx;
    uint64_t num = vq->num;
    used_ring->flags = vq->used_flags;
    used_ring->ring[used_idx % num].id = desc_head_idx;
    used_ring->ring[used_idx % num].len = req_len;
    log_debug("used_ring->idx is %d\n", used_ring->idx);
    used_ring->idx++;
    log_debug("changed used_ring->idx is %d\n", used_ring->idx);
}

void virtqueue_disable_notify(VirtQueue *vq) {
    vq->used_ring->flags |= (uint16_t)VRING_USED_F_NO_NOTIFY;
}

void virtqueue_enable_notify(VirtQueue *vq) {
    vq->used_ring->flags &= !(uint16_t)VRING_USED_F_NO_NOTIFY;
}

int virtio_blk_notify_handler(VirtIODevice *vdev, VirtQueue *vq)
{
    log_trace("virtio blk notify handler enter");
    /*
    1. 从可用环中取出请求, 
    2. 将请求池的各个请求映射为文件进行处理
    */
    while(!virtqueue_is_empty(vq)) {
        uint16_t desc_idx = virtqueue_pop_desc_chain_head(vq); //描述符链头
        // TODO: 这个notify是怎么弄???
        virtqueue_disable_notify(vq);
        // if (vq->avail_ring->idx == vq->last_avail_idx) {
        // }
        log_debug("avail_idx is %d, last_avail_idx is %d, desc_head_idx is %d", vq->avail_ring->idx, vq->last_avail_idx, desc_idx);
        virtqueue_handle_request(vq, desc_idx);
        virtqueue_enable_notify(vq);
    }
    return 0;
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