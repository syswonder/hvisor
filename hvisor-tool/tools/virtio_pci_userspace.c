// SPDX-License-Identifier: GPL-2.0-only
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

#include "hvisor.h"
#include "log.h"
#include "virtio_internal.h"
#include "virtio_pci_userspace.h"

static volatile struct virtio_pci_bridge *virtio_pci_bridge;
static pthread_t virtio_pci_userspace_thread;
static atomic_bool virtio_pci_userspace_stop;
static bool virtio_pci_userspace_thread_started;
static unsigned int userspace_device_mask;
static UserspacePciDev userspace_pci_devs[MAX_VIRTPCI_DEV];
static int next_userspace_pci_dev = MAX_VIRTPCI_DEV - 1;

uint32_t userspace_pci_le32_to_cpu(uint32_t value) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return value;
#else
    return __builtin_bswap32(value);
#endif
}

uint64_t userspace_pci_le64_to_cpu(uint64_t value) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return value;
#else
    return __builtin_bswap64(value);
#endif
}

size_t userspace_pci_virtio_avail_ring_size(__u64 queue_size) {
    return sizeof(__u16) * 2 + sizeof(__u16) * queue_size;
}

size_t userspace_pci_virtio_used_ring_size(__u64 queue_size) {
    return sizeof(__u16) * 2 + sizeof(struct virtq_used_elem) * queue_size;
}

int userspace_pci_map_phys_region(__u64 phys, size_t len, MappedRegion *region,
                                  void **mapped_ptr) {
    long page_size = sysconf(_SC_PAGESIZE);
    __u64 page_mask;
    __u64 aligned;
    size_t offset;
    size_t map_len;
    void *base;

    if (page_size <= 0)
        return -1;

    page_mask = ~((__u64)page_size - 1);
    aligned = phys & page_mask;
    offset = (size_t)(phys - aligned);
    map_len = offset + len;
    map_len = (map_len + page_size - 1) & ~(page_size - 1);
    base = mmap(NULL, map_len, PROT_READ | PROT_WRITE, MAP_SHARED, ko_fd,
                (off_t)aligned);
    if (base == MAP_FAILED)
        return -1;

    region->base = base;
    region->len = map_len;
    *mapped_ptr = (char *)base + offset;
    return 0;
}

void userspace_pci_unmap_region(MappedRegion *region) {
    if (region->base != NULL) {
        munmap(region->base, region->len);
        region->base = NULL;
        region->len = 0;
    }
}

static bool region_contains(uint64_t base, uint64_t size, uint64_t addr,
                            size_t len) {
    return addr >= base && addr - base < size && len <= size - (addr - base);
}

static int get_ring_zone(uint64_t addr, size_t len) {
    int zone_id = -1;

    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    for (int zone = 0; zone < MAX_ZONES; zone++) {
        const struct zone_mem *memory = &zone_mem[zone];
        for (size_t i = 0; i < memory->num_regions; i++) {
            const struct zone_mem_region *region = &memory->regions[i];
            if (region_contains(region->zone0_ipa, region->mem_size, addr,
                                len)) {
                if (zone_id >= 0 && zone_id != zone) {
                    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
                    return -EINVAL;
                }
                zone_id = zone;
            }
        }
    }
    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    return zone_id >= 0 ? zone_id : -ENOENT;
}

int userspace_pci_translate_addr(int zone_id, __u64 ipa, size_t len,
                                 void **mapped_ptr) {
    int ret = -ENOENT;

    pthread_mutex_lock(&ZONE_MEM_MUTEX);
    const struct zone_mem *memory = &zone_mem[zone_id];
    for (size_t i = 0; i < memory->num_regions; i++) {
        const struct zone_mem_region *region = &memory->regions[i];
        if (region_contains(region->zonex_ipa, region->mem_size, ipa, len)) {
            *mapped_ptr =
                (void *)(region->virt_addr + (ipa - region->zonex_ipa));
            ret = 0;
            break;
        }
    }
    pthread_mutex_unlock(&ZONE_MEM_MUTEX);
    return ret;
}

static void virtio_pci_userspace_notify(__u16 op, __u32 target_cpu,
                                        __u32 request_id, __u32 status) {
    struct virtio_pci_hypercall_info info = {
        .version = VIRTIO_PCI_HYPERCALL_VERSION,
        .op = op,
        .target_cpu = target_cpu,
        .request_id = request_id,
        .status = status,
    };

    if (ioctl(ko_fd, HVISOR_VIRTIO_PCI_NOTIFY, &info) < 0)
        log_error("VirtIO PCI completion ioctl failed: %s", strerror(errno));
}

static bool userspace_pci_config_res_full(void) {
    __u32 front = __atomic_load_n(&virtio_pci_bridge->config_res_front,
                                  memory_order_acquire);
    __u32 rear = __atomic_load_n(&virtio_pci_bridge->config_res_rear,
                                 memory_order_relaxed);

    return ((rear + 1) & (MAX_PCI_CONFIG_RES - 1)) == front;
}

static void userspace_pci_push_config_res(__u32 request_id, __u32 status,
                                          __u16 dev_id) {
    __u32 rear = __atomic_load_n(&virtio_pci_bridge->config_res_rear,
                                 memory_order_relaxed);
    struct virtio_pci_config_res *res;

    res = (struct virtio_pci_config_res *)&virtio_pci_bridge
              ->config_res_list[rear];
    res->request_id = request_id;
    res->status = status;
    res->dev_id = dev_id;
    res->padding = 0;
    __atomic_store_n(&virtio_pci_bridge->config_res_rear,
                     (rear + 1) & (MAX_PCI_CONFIG_RES - 1),
                     memory_order_release);
}

static void destroy_userspace_pci_dev(UserspacePciDev *dev) {
    int i;

    switch (dev->dtype) {
    case VIRTIO_PCI_BLK:
        userspace_pci_blk_cleanup_dev(dev);
        break;
    case VIRTIO_PCI_RNG:
        userspace_pci_rng_cleanup_dev(dev);
        break;
    default:
        break;
    }

    for (i = 0; i < dev->num_of_vq; i++) {
        userspace_pci_unmap_region(&dev->vqs[i].desc_region);
        userspace_pci_unmap_region(&dev->vqs[i].avail_region);
        userspace_pci_unmap_region(&dev->vqs[i].used_region);
        dev->vqs[i].desc = NULL;
        dev->vqs[i].avail = NULL;
        dev->vqs[i].used = NULL;
        dev->vqs[i].queue_size = 0;
    }
    dev->dtype = 0;
    dev->num_of_vq = 0;
    dev->active = false;
}

static int create_userspace_pci_dev(const struct virtio_pci_config_info *info,
                                    __u16 *dev_id) {
    UserspacePciDev *dev;
    int allocated_dev;
    int i;
    int ret = -1;

    if (info->num_of_queues == 0 || info->num_of_queues > MAX_VQ)
        return -EINVAL;
    if (info->dtype != VIRTIO_PCI_RNG && info->dtype != VIRTIO_PCI_BLK)
        return -EOPNOTSUPP;
    if (next_userspace_pci_dev < VIRTIO_PCI_USERSPACE_DEV_BASE)
        return -ENOSPC;

    allocated_dev = next_userspace_pci_dev--;
    *dev_id = (__u16)allocated_dev;
    dev = &userspace_pci_devs[*dev_id];
    memset(dev, 0, sizeof(*dev));
    dev->dtype = info->dtype;
    dev->num_of_vq = info->num_of_queues;
    dev->backend.blk.image_fd = -1;

    for (i = 0; i < dev->num_of_vq; i++) {
        size_t desc_len;
        size_t avail_len;
        size_t used_len;

        uint64_t size = info->vqs[i].queue_size;
        int zone_id;

        if (!size || size > UINT16_MAX || (size & (size - 1)))
            goto err_out;

        dev->vqs[i].queue_size = (__u16)info->vqs[i].queue_size;
        desc_len = sizeof(struct virtq_desc) * dev->vqs[i].queue_size;
        avail_len =
            userspace_pci_virtio_avail_ring_size(dev->vqs[i].queue_size);
        used_len = userspace_pci_virtio_used_ring_size(dev->vqs[i].queue_size);

        zone_id = get_ring_zone(info->vqs[i].desc_area, desc_len);
        if (zone_id < 0 ||
            get_ring_zone(info->vqs[i].avail_area, avail_len) != zone_id ||
            get_ring_zone(info->vqs[i].used_area, used_len) != zone_id)
            goto err_out;
        dev->vqs[i].zone_id = zone_id;

        if (userspace_pci_map_phys_region(info->vqs[i].desc_area, desc_len,
                                          &dev->vqs[i].desc_region,
                                          (void **)&dev->vqs[i].desc) != 0 ||
            userspace_pci_map_phys_region(info->vqs[i].avail_area, avail_len,
                                          &dev->vqs[i].avail_region,
                                          (void **)&dev->vqs[i].avail) != 0 ||
            userspace_pci_map_phys_region(info->vqs[i].used_area, used_len,
                                          &dev->vqs[i].used_region,
                                          (void **)&dev->vqs[i].used) != 0) {
            goto err_out;
        }
    }

    switch (dev->dtype) {
    case VIRTIO_PCI_BLK:
        ret = userspace_pci_blk_init_dev(dev, info);
        break;
    case VIRTIO_PCI_RNG:
        ret = userspace_pci_rng_init_dev(dev, info);
        break;
    default:
        ret = -EOPNOTSUPP;
        break;
    }
    if (ret != 0)
        goto err_out;

    dev->active = true;
    return 0;

err_out:
    destroy_userspace_pci_dev(dev);
    next_userspace_pci_dev = allocated_dev;
    return ret;
}

static bool handle_userspace_pci_config(void) {
    __u32 front;
    __u32 rear;
    bool did_work = false;

    front = __atomic_load_n(&virtio_pci_bridge->config_req_front,
                            memory_order_acquire);
    rear = __atomic_load_n(&virtio_pci_bridge->config_req_rear,
                           memory_order_acquire);
    while (front != rear) {
        struct virtio_pci_config_req req;
        __u16 dev_id = 0;
        __u32 status = 0;
        int ret;

        memcpy(&req, (const void *)&virtio_pci_bridge->config_req_list[front],
               sizeof(req));
        if (!((req.info.dtype == VIRTIO_PCI_RNG &&
               (userspace_device_mask & VIRTIO_PCI_USERSPACE_RNG)) ||
              (req.info.dtype == VIRTIO_PCI_BLK &&
               (userspace_device_mask & VIRTIO_PCI_USERSPACE_BLK))) ||
            userspace_pci_config_res_full())
            break;

        ret = create_userspace_pci_dev(&req.info, &dev_id);
        if (ret != 0)
            status = (__u32)(-ret);
        userspace_pci_push_config_res(req.request_id, status, dev_id);

        __atomic_store_n(&virtio_pci_bridge->config_req_front,
                         (front + 1) & (MAX_PCI_CONFIG_REQ - 1),
                         memory_order_release);
        virtio_pci_userspace_notify(VIRTIO_PCI_HC_OP_CONFIG_REQ_COMPLETE, 0,
                                    req.request_id, status);
        did_work = true;
        front = __atomic_load_n(&virtio_pci_bridge->config_req_front,
                                memory_order_acquire);
        rear = __atomic_load_n(&virtio_pci_bridge->config_req_rear,
                               memory_order_acquire);
    }

    return did_work;
}

static bool handle_userspace_pci_data(void) {
    __u32 front;
    __u32 rear;
    bool did_work = false;

    front = __atomic_load_n(&virtio_pci_bridge->data_req_front,
                            memory_order_acquire);
    rear = __atomic_load_n(&virtio_pci_bridge->data_req_rear,
                           memory_order_acquire);
    while (front != rear) {
        struct virtio_pci_data_req req;
        UserspacePciDev *dev;
        int ret;
        __u32 status = 0;

        memcpy(&req, (const void *)&virtio_pci_bridge->data_req_list[front],
               sizeof(req));
        if (req.info.dev_id < VIRTIO_PCI_USERSPACE_DEV_BASE) {
            log_debug(
                "userspace pci backend: dev_id %u belongs to kernel backend",
                req.info.dev_id);
            break;
        }
        __atomic_store_n(&virtio_pci_bridge->data_req_front,
                         (front + 1) & (MAX_PCI_DATA_REQ - 1),
                         memory_order_release);

        if (req.info.dev_id >= MAX_VIRTPCI_DEV) {
            log_error("userspace pci backend: invalid dev_id %u out of range",
                      req.info.dev_id);
            status = EINVAL;
            goto complete;
        }
        dev = &userspace_pci_devs[req.info.dev_id];
        if (!dev->active || req.info.queue_id >= dev->num_of_vq) {
            log_error(
                "userspace pci backend: inactive dev %u or invalid queue %u/%u",
                req.info.dev_id, req.info.queue_id, dev->num_of_vq);
            status = EINVAL;
            goto complete;
        }

        switch (dev->dtype) {
        case VIRTIO_PCI_RNG:
            ret = userspace_pci_rng_process_vq(&dev->vqs[req.info.queue_id]);
            break;
        case VIRTIO_PCI_BLK:
            ret =
                userspace_pci_blk_process_vq(dev, &dev->vqs[req.info.queue_id]);
            break;
        default:
            ret = -EOPNOTSUPP;
            break;
        }
        if (ret != 0) {
            log_error("userspace pci backend: failed to process dev %u type %u "
                      "queue %u, ret=%d",
                      req.info.dev_id, dev->dtype, req.info.queue_id, ret);
            status = (__u32)(-ret);
        }

    complete:
        virtio_pci_userspace_notify(VIRTIO_PCI_HC_OP_DATA_REQ_COMPLETE,
                                    req.info.cpu_id, req.request_id, status);
        did_work = true;
        front = __atomic_load_n(&virtio_pci_bridge->data_req_front,
                                memory_order_acquire);
        rear = __atomic_load_n(&virtio_pci_bridge->data_req_rear,
                               memory_order_acquire);
    }

    return did_work;
}

static void *virtio_pci_userspace_thread_main(void *arg) {
    static const long min_sleep_ns = 50000;
    static const long max_sleep_ns = 10000000;
    static const unsigned int idle_backoff_shift = 1;
    long sleep_ns = min_sleep_ns;

    (void)arg;

    while (!atomic_load_explicit(&virtio_pci_userspace_stop,
                                 memory_order_relaxed)) {
        bool did_work;

        did_work = handle_userspace_pci_config();
        did_work = handle_userspace_pci_data() || did_work;
        if (did_work) {
            sleep_ns = min_sleep_ns;
            continue;
        }

        nanosleep(&(struct timespec){.tv_sec = 0, .tv_nsec = sleep_ns}, NULL);
        if (sleep_ns < max_sleep_ns) {
            sleep_ns <<= idle_backoff_shift;
            if (sleep_ns > max_sleep_ns)
                sleep_ns = max_sleep_ns;
        }
    }
    return NULL;
}

int virtio_pci_userspace_backend_init(void) {
    struct hvisor_virtio_pci_info info;
    long page_size = sysconf(_SC_PAGESIZE);
    int ret;

    if (ioctl(ko_fd, HVISOR_GET_VIRTIO_PCI_INFO, &info) < 0)
        return errno == ENODEV || errno == ENOTTY ? 0 : -errno;
    if (!info.userspace_devices)
        return 0;
    if (page_size <= 0)
        return -EINVAL;
    userspace_device_mask = info.userspace_devices;
    virtio_pci_bridge =
        mmap(NULL, VIRTIO_PCI_BRIDGE_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED,
             ko_fd, (off_t)(info.pfn * page_size));
    if ((void *)virtio_pci_bridge == MAP_FAILED) {
        virtio_pci_bridge = NULL;
        return -errno;
    }

    atomic_store_explicit(&virtio_pci_userspace_stop, false,
                          memory_order_relaxed);
    ret = pthread_create(&virtio_pci_userspace_thread, NULL,
                         virtio_pci_userspace_thread_main, NULL);
    if (ret != 0) {
        munmap((void *)virtio_pci_bridge, VIRTIO_PCI_BRIDGE_SIZE);
        virtio_pci_bridge = NULL;
        return -ret;
    }

    virtio_pci_userspace_thread_started = true;
    log_info("VirtIO PCI userspace backend enabled");
    return 0;
}

void virtio_pci_userspace_backend_shutdown(void) {
    int i;

    if (virtio_pci_userspace_thread_started) {
        atomic_store_explicit(&virtio_pci_userspace_stop, true,
                              memory_order_relaxed);
        pthread_join(virtio_pci_userspace_thread, NULL);
        virtio_pci_userspace_thread_started = false;
    }

    for (i = 0; i < MAX_VIRTPCI_DEV; i++)
        destroy_userspace_pci_dev(&userspace_pci_devs[i]);

    if (virtio_pci_bridge != NULL) {
        munmap((void *)virtio_pci_bridge, VIRTIO_PCI_BRIDGE_SIZE);
        virtio_pci_bridge = NULL;
    }

    next_userspace_pci_dev = MAX_VIRTPCI_DEV - 1;
}
