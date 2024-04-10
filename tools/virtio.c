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
#include <getopt.h>

/// hvisor kernel module fd
int ko_fd;
volatile struct hvisor_device_region *device_region;

pthread_mutex_t RES_MUTEX = PTHREAD_MUTEX_INITIALIZER;
// TODO: 改成链表
VirtIODevice *vdevs[16];
int vdevs_num;

void *virt_addr;
void *phys_addr;
#define NON_ROOT_PHYS_START 0x70000000
#define NON_ROOT_PHYS_SIZE 0x20000000

inline int is_queue_full(unsigned int front, unsigned int rear, unsigned int size)
{
    if (((rear + 1) & (size - 1)) == front) {
        log_trace("queue is full, front is %u, rear is %u", front, rear);
        return 1;
    } else {
        return 0;
    }
}

inline int is_queue_empty(unsigned int front, unsigned int rear)
{
    return rear == front;
}

// create a virtio device.
static VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t zone_id, 
						uint64_t base_addr, uint64_t len, uint32_t irq_id, void* arg)
{
	log_info("create virtio device type %d, zone id %d, base addr %lx, len %lx, irq id %d", 
				dev_type, zone_id, base_addr, len, irq_id);
    VirtIODevice *vdev = NULL;
	vdev = calloc(1, sizeof(VirtIODevice));
	init_mmio_regs(&vdev->regs, dev_type);
	vdev->base_addr = base_addr;
	vdev->len = len;
	vdev->zone_id = zone_id;
	vdev->irq_id = irq_id;
	vdev->type = dev_type;
    switch (dev_type)
    {
    case VirtioTBlock: {
		int img_fd = open((const char*)arg, O_RDWR);
        if (img_fd == -1) {
			log_error("cannot open %s, Error code is %d\n", (char*)arg, errno);
			goto err;
		}
        vdev->regs.dev_feature = VIRTIO_BLK_F_SEG_MAX | VIRTIO_BLK_F_SIZE_MAX | VIRTIO_F_VERSION_1;
        vdev->dev = init_blk_dev(vdev, BLK_SIZE_MAX, img_fd); // 256MB
        init_virtio_queue(vdev, dev_type);
        break;
	}
    case VirtioTNet:
		// TODO: add VIRTIO_NET_F_MRG_RXBUF?
        vdev->regs.dev_feature = VIRTIO_F_VERSION_1 | VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS;
        vdev->type = dev_type;
        uint8_t mac[] = {0x00, 0x16, 0x3E, 0x10, 0x10, 0x10};
        vdev->dev = init_net_dev(mac);
        init_virtio_queue(vdev, dev_type);
        virtio_net_init(vdev, (char *)arg);
        break;
	default:
		log_error("unsupported virtio device type\n");
		goto err;
    }
    vdevs[vdevs_num++] = vdev;
    return vdev;

err:
	free(vdev);
	return NULL;
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
        vq->dev = vdev;
        vdev->vqs = vq;
        break;
    case VirtioTNet:
        vdev->vqs_len = NET_MAX_QUEUES;
        vq = malloc(sizeof(VirtQueue) * NET_MAX_QUEUES);
        for (int i = 0; i < NET_MAX_QUEUES; ++i) {
            virtqueue_reset(vq, i);
            vq[i].queue_num_max = VIRTQUEUE_NET_MAX_SIZE;
            vq[i].dev = vdev;
        }
        vq[NET_QUEUE_RX].notify_handler = virtio_net_rxq_notify_handler;
        vq[NET_QUEUE_TX].notify_handler = virtio_net_txq_notify_handler;
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

void virtio_dev_reset(VirtIODevice *vdev)
{
    // When driver read first 4 encoded messages, it will reset dev.
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
    // reserve these fields
    void *addr = vq->notify_handler;
    VirtIODevice *dev = vq->dev;
    uint32_t queue_num_max = vq->queue_num_max;
    memset(vq, 0, sizeof(VirtQueue));
    vq->vq_idx = idx;
    vq->notify_handler = addr;
    vq->dev = dev;
    vq->queue_num_max = queue_num_max;
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

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx)
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

// When virtio device is processing virtqueue, driver adding an elem to virtqueue is no need to notify device.
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

// record one descriptor to iov.
static inline int _descriptor2iov(int i, volatile VirtqDesc *vd,
           struct iovec *iov, int max_iov, uint16_t *flags) {
	if (i >= max_iov)
        return -1;

    void *host_addr;
    host_addr = get_virt_addr(vd->addr);
    iov[i].iov_base = host_addr;
    iov[i].iov_len = vd->len;
    // log_debug("vd->addr ipa is %x, iov_base is %x, iov_len is %d", vd->addr, host_addr, vd->len);
    if (flags != NULL)
        flags[i] = vd->flags;
    return 0;
}

/// record one descriptor list to iov
/// \param desc_idx the first descriptor's idx in descriptor list.
/// \param max_iov the max num of iov
/// \param flags each descriptor's flags
/// \return the valid num of iov
int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                struct iovec *iov, int max_iov, uint16_t *flags)
{
    uint16_t next, idx;
    volatile VirtqDesc *vdesc;
    idx = vq->last_avail_idx;
    if(idx == vq->avail_ring->idx)
        return 0;
    vq->last_avail_idx++;
    *desc_idx = next = vq->avail_ring->ring[idx & (vq->num - 1)];

    for (int i=0; i < vq->num; next = vdesc->next) {
        vdesc = &vq->desc_table[next];
        if (_descriptor2iov(i, vdesc, iov, max_iov, flags)) {
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
    elem = &used_ring->ring[used_idx++ & mask];
    elem->id = idx;
    elem->len = iolen;
    used_ring->idx = used_idx;
    log_debug("update used ring: used_idx is %d, elem->idx is %d", used_idx-1, idx);
}

/// If vq's used ring is changed, then inject interrupt to vq's zone
void vq_finish_chain(VirtQueue *vq, int no_more_chains)
{
    if (!no_more_chains) {
        return;
    }
    uint16_t new_idx, old_idx;
    int need_interrupt;
    old_idx = vq->last_used_idx;
    vq->last_used_idx = new_idx = vq->used_ring->idx;
    need_interrupt = new_idx != old_idx && !(vq->avail_ring->flags & VRING_AVAIL_F_NO_INTERRUPT);
    if (need_interrupt)
        virtio_inject_irq(vq->dev->zone_id, vq->dev->irq_id);
}

static uint64_t virtio_mmio_read(VirtIODevice *vdev, uint64_t offset, unsigned size)
{
    log_debug("virtio mmio read at %#x", offset);
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
    log_debug("virtio mmio write at %#x, value is %#x\n", offset, value);
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

/// Write barrier to make sure all write operations are finished before this operation
static inline void dmb_ishst(void) {
    asm volatile ("dmb ishst":: : "memory");
}

// Inject irq_id to target zone. It will add to res list, and notify hypervisor through ioctl.
void virtio_inject_irq(uint32_t target_zone, uint32_t irq_id)
{
    volatile struct device_res *res;
    while (is_queue_full(device_region->res_front, device_region->res_rear, MAX_REQ));
    pthread_mutex_lock(&RES_MUTEX);
    unsigned int res_rear = device_region->res_rear;
    res = &device_region->res_list[res_rear];
    res->irq_id = irq_id;
    res->target_zone = target_zone;
    dmb_ishst();
    device_region->res_rear = (res_rear + 1) & (MAX_REQ - 1);
    pthread_mutex_unlock(&RES_MUTEX);
    ioctl(ko_fd, HVISOR_FINISH);
}

static void virtio_finish_cfg_req(uint32_t target_cpu, uint64_t value) {
    device_region->cfg_values[target_cpu] = value;
    dmb_ishst();
    device_region->cfg_flags[target_cpu]++;
    dmb_ishst();
}

int virtio_handle_req(volatile struct device_req *req)
{
    int i;
    uint64_t value = 0;
    for (i = 0; i < vdevs_num; ++i) {
        if ((req->src_zone == vdevs[i]->zone_id) && in_range(req->address, vdevs[i]->base_addr, vdevs[i]->len))
            break;
    }
    if (i == vdevs_num) {
        log_error("no matched virtio dev");
        return -1;
    }
    VirtIODevice *vdev = vdevs[i];
    if (vdev->type == VirtioTNet)
        log_debug("vdev type is net");
    else
        log_debug("vdev type is blk");
    uint64_t offs = req->address - vdev->base_addr;
    if (req->is_write) {
        virtio_mmio_write(vdev, offs, req->value, req->size);
    } else {
        value = virtio_mmio_read(vdev, offs, req->size);
        log_debug("read value is 0x%x\n", value);
    }
    if (!req->need_interrupt) {
        // If a request is a control not a data request
        virtio_finish_cfg_req(req->src_cpu, value);
    } 
    log_trace("src_zone is %d, src_cpu is %lld", req->src_zone, req->src_cpu);
    return 0;
}

void handle_virtio_requests()
{
    unsigned int req_front = device_region->req_front;
    volatile struct device_req *req;
    while (1) {
        if (!is_queue_empty(req_front, device_region->req_rear)) {
            req = &device_region->req_list[req_front];
            virtio_handle_req(req);
            req_front = (req_front + 1) & (MAX_REQ - 1);
            device_region->req_front = req_front;
            // dmb_ishst(); 应该不需要
        }
    }
}

int virtio_init()
{
    // The higher log level is , faster virtio-blk will be.
    int err;

    log_set_level(LOG_DEBUG);
    FILE *log_file = fopen("log.txt", "w+");
    if (log_file == NULL) {
        log_error("open log file failed");
    }
    log_add_fp(log_file, LOG_DEBUG);
    log_info("hvisor init");
    ko_fd = open("/dev/hvisor", O_RDWR);
    if (ko_fd < 0) {
        log_error("open hvisor failed");
        exit(1);
    }
    // ioctl for init virtio
    err = ioctl(ko_fd, HVISOR_INIT_VIRTIO);
    if (err) {
        log_error("ioctl failed, err code is %d", err);
        close(ko_fd);
        exit(1);
    }

    // mmap: create shared memory
    device_region = (struct hvisor_device_region *) mmap(NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, ko_fd, 0);
    if (device_region == (void *)-1) {
        log_error("mmap failed");
        goto unmap;
    }

	// mmap: map non root linux physical memory to virtual memory
	// TODO：根据配置文件
    int mem_fd = open("/dev/mem", O_RDWR | O_SYNC);
    phys_addr = NON_ROOT_PHYS_START;
    virt_addr = mmap(NULL, NON_ROOT_PHYS_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, mem_fd, (off_t) phys_addr);
	close(mem_fd);
    log_info("mmap virt addr is %#x", virt_addr);

    initialize_event_monitor();
    log_info("hvisor init okay!");
unmap:
    munmap((void *)device_region, MMAP_SIZE);
    return 0;
}

static int create_virtio_device_from_cmd(char *cmd) {
	log_info("cmd is %s", cmd);
	VirtioDeviceType dev_type = VirtioTNone;
	uint64_t base_addr = 0, len = 0;
	uint32_t zone_id = 0, irq_id = 0;
	char *opt, *now, *arg = NULL;

	opt = strdup(cmd);
	now = strtok(opt, ",");

	if (strcmp(now, "blk") == 0) {
		dev_type = VirtioTBlock;
	} else if (strcmp(now, "net") == 0) {
		dev_type = VirtioTNet;
	} else {
		log_error("unknown device type %s", now);
		return -1;
	}

	while ((now = strtok(NULL, "=")) != NULL) {
		if (strcmp(now, "addr") == 0) {
			now = strtok(NULL, ",");
			base_addr = strtoul(now, NULL, 16);
		} else if (strcmp(now, "len") == 0) {
			now = strtok(NULL, ",");
			len = strtoul(now, NULL, 16);
		} else if (strcmp(now, "irq") == 0) {
			now = strtok(NULL, ",");
			irq_id = strtoul(now, NULL, 10);
		} else if (strcmp(now, "zone_id") == 0) {
			now = strtok(NULL, ",");
			zone_id = strtoul(now, NULL, 10);
		} else if (strcmp(now, "img") == 0) {
			if (dev_type != VirtioTBlock) {
				log_error("image path only for block device");
				return -1;
			}
			arg = strtok(NULL, ",");
		} else if (strcmp(now, "tap") == 0) {
			if (dev_type != VirtioTNet) {
				log_error("tap only for net device");
				return -1;
			}
			arg = strtok(NULL, ",");
		} else {
			log_error("unknown option %s", now);
			return -1;
		}
	}
	free(opt);

	if (base_addr == 0 || len == 0 || irq_id == 0 || zone_id == 0) {
		printf("missing arguments");
		return -1;
	}
	create_virtio_device(dev_type, zone_id, base_addr, len, irq_id, arg);
	return 0;
}

int virtio_start(int argc, char *argv[]) {
	static struct option long_options[] = {
		{"device", required_argument, 0, 'd'},	
		{0, 0, 0, 0}
	};
	char *optstring = "d:";
	int opt, err = 0;

	virtio_init();
	while ( (opt = getopt_long(argc, argv, optstring, long_options, NULL)) != -1) {
		switch (opt) {
			case 'd':
				err = create_virtio_device_from_cmd(optarg);
				if (err) {
					log_error("create virtio device failed");
					goto err_out;
				}
				break;
			default:
				log_error("unknown option %c", opt);
				goto err_out;
		}
	}
    handle_virtio_requests();

err_out:
	// TODO: shutdown virtio devices
	return err;
}

