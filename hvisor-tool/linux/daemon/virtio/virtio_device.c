// SPDX-License-Identifier: GPL-2.0-only
#include <errno.h>
#include <pthread.h>
#include <stdlib.h>

#include "log.h"
#include "virtio_blk.h"
#include "virtio_console.h"
#include "virtio_internal.h"
#include "virtio_net.h"
#include "virtio_scmi.h"
#ifdef ENABLE_VIRTIO_GPU
#include "virtio_gpu.h"
#endif

pthread_mutex_t VDEV_MUTEX = PTHREAD_MUTEX_INITIALIZER;
VirtIODevice *vdevs[MAX_DEVS];
int vdevs_num;

static const struct virtio_device_ops *const device_ops_table[] = {
    [VirtioTBlock] = &virtio_blk_ops,       [VirtioTNet] = &virtio_net_ops,
    [VirtioTConsole] = &virtio_console_ops, [VirtioTSCMI] = &virtio_scmi_ops,
#ifdef ENABLE_VIRTIO_GPU
    [VirtioTGPU] = &virtio_gpu_ops,
#endif
};

const struct virtio_device_ops *virtio_lookup_ops(VirtioDeviceType type) {
    int n = (int)(sizeof(device_ops_table) / sizeof(device_ops_table[0]));
    if (type <= VirtioTNone || (int)type >= n)
        return NULL;
    return device_ops_table[type];
}

static int init_virtio_queue(VirtIODevice *vdev,
                             const struct virtio_device_ops *ops) {
    log_info("Initializing virtio queue for zone:%d, device type:%s",
             vdev->zone_id, virtio_device_type_to_string(ops->type));

    if (ops->num_queues == 0 || ops->num_queues > VIRTIO_MAX_VQUEUES) {
        log_error("invalid queue count %u for %s", ops->num_queues,
                  virtio_device_type_to_string(ops->type));
        return -EINVAL;
    }

    vdev->vqs_len = ops->num_queues;
    VirtQueue *vqs = calloc(ops->num_queues, sizeof(VirtQueue));
    if (!vqs)
        return -ENOMEM;

    for (uint32_t i = 0; i < ops->num_queues; i++) {
        virtqueue_reset(&vqs[i], i);
        vqs[i].queue_num_max = ops->queue_max_size;
        vqs[i].dev = vdev;
        if (ops->notify_handlers[i])
            vqs[i].notify_handler = ops->notify_handlers[i];
    }
    vdev->vqs = vqs;
    return 0;
}

void destroy_unpublished_virtio_device(VirtIODevice *vdev) {
    if (!vdev)
        return;

    if (vdev->virtio_close) {
        vdev->virtio_close(vdev);
        return;
    }

    free(vdev->vqs);
    free(vdev);
}

int publish_virtio_devices(VirtIODevice **new_devs, size_t count) {
    if (count == 0)
        return 0;

    pthread_mutex_lock(&VDEV_MUTEX);
    if (count > MAX_DEVS - (size_t)vdevs_num) {
        pthread_mutex_unlock(&VDEV_MUTEX);
        log_error("virtio device num exceed max limit");
        return -1;
    }

    for (size_t i = 0; i < count; i++)
        vdevs[vdevs_num++] = new_devs[i];
    pthread_mutex_unlock(&VDEV_MUTEX);
    return 0;
}

VirtIODevice *create_virtio_device_unpublished(
    VirtioDeviceType dev_type, uint32_t zone_id, uint64_t base_addr,
    uint64_t len, uint32_t irq_id, const VirtioDeviceParams *params) {
    const struct virtio_device_ops *ops = virtio_lookup_ops(dev_type);
    if (!ops) {
        log_error("unsupported virtio device type %d", dev_type);
        return NULL;
    }

    log_info(
        "create virtio device type %s, zone id %d, base addr %lx, len %lx, "
        "irq id %d",
        virtio_device_type_to_string(dev_type), zone_id, base_addr, len,
        irq_id);

    VirtIODevice *vdev = calloc(1, sizeof(VirtIODevice));
    if (!vdev) {
        log_error("failed to allocate virtio device");
        return NULL;
    }

    init_mmio_regs(&vdev->regs, dev_type);
    vdev->base_addr = base_addr;
    vdev->len = len;
    vdev->zone_id = zone_id;
    vdev->irq_id = irq_id;
    vdev->type = dev_type;
    pthread_mutex_init(&vdev->interrupt_lock, NULL);
    vdev->interrupt_line_asserted = false;
    vdev->regs.dev_feature = ops->features;
    vdev->virtio_close = ops->close;
    vdev->status_changed = ops->status_changed;

    if (init_virtio_queue(vdev, ops) != 0)
        goto err;

    if (ops->init(vdev, params) != 0)
        goto err;

    log_info("create %s success", virtio_device_type_to_string(dev_type));
    return vdev;

err:
    ops->close(vdev);
    return NULL;
}

VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t zone_id,
                                   uint64_t base_addr, uint64_t len,
                                   uint32_t irq_id,
                                   const VirtioDeviceParams *params) {
    VirtIODevice *vdev = create_virtio_device_unpublished(
        dev_type, zone_id, base_addr, len, irq_id, params);
    if (!vdev)
        return NULL;

    if (publish_virtio_devices(&vdev, 1) != 0) {
        destroy_unpublished_virtio_device(vdev);
        return NULL;
    }
    return vdev;
}

void virtio_destroy_all_devices(void) {
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; i++) {
        if (vdevs[i] && vdevs[i]->virtio_close)
            vdevs[i]->virtio_close(vdevs[i]);
        vdevs[i] = NULL;
    }
    vdevs_num = 0;
    pthread_mutex_unlock(&VDEV_MUTEX);
}
