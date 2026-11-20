// SPDX-License-Identifier: GPL-2.0-only
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>

#include "json_parse.h"
#include "loader.h"
#include "log.h"
#include "safe_cjson.h"
#include "virtio_blk.h"
#include "virtio_console.h"
#include "virtio_internal.h"
#include "virtio_net.h"
#include "virtio_scmi.h"
#ifdef ENABLE_VIRTIO_GPU
#include "virtio_gpu.h"
#endif

static const struct virtio_config_ops *const config_ops_table[] = {
    [VirtioTBlock] = &virtio_blk_config_ops,
    [VirtioTNet] = &virtio_net_config_ops,
    [VirtioTConsole] = &virtio_console_config_ops,
    [VirtioTSCMI] = &virtio_scmi_config_ops,
#ifdef ENABLE_VIRTIO_GPU
    [VirtioTGPU] = &virtio_gpu_config_ops,
#endif
};

const struct virtio_config_ops *
virtio_lookup_config_ops(VirtioDeviceType type) {
    int n = (int)(sizeof(config_ops_table) / sizeof(config_ops_table[0]));
    if (type <= VirtioTNone || (int)type >= n)
        return NULL;
    return config_ops_table[type];
}

static int parse_virtio_device_type(const char *type,
                                    VirtioDeviceType *dev_type) {
    if (strcmp(type, "blk") == 0)
        *dev_type = VirtioTBlock;
    else if (strcmp(type, "net") == 0)
        *dev_type = VirtioTNet;
    else if (strcmp(type, "console") == 0)
        *dev_type = VirtioTConsole;
    else if (strcmp(type, "gpu") == 0)
        *dev_type = VirtioTGPU;
    else if (strcmp(type, "scmi") == 0)
        *dev_type = VirtioTSCMI;
    else {
        log_error("unknown device type %s", type);
        return -1;
    }
    return 0;
}

static void free_virtio_device_config(VirtioDeviceConfig *cfg) {
    if (cfg->config_ops && cfg->config_ops->free)
        cfg->config_ops->free(&cfg->params);
    memset(&cfg->params, 0, sizeof(cfg->params));
}

static void free_virtio_config(VirtioConfig *cfg) {
    for (size_t zi = 0; zi < cfg->zone_num; zi++)
        for (size_t di = 0; di < cfg->zones[zi].device_num; di++)
            free_virtio_device_config(&cfg->zones[zi].devices[di]);
}

static int parse_virtio_device_config(const cJSON *device_json,
                                      uint32_t zone_id,
                                      VirtioDeviceConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cfg->zone_id = zone_id;

    cJSON *status = SAFE_CJSON_GET_OBJECT_ITEM(device_json, "status");
    if (!cJSON_IsString(status)) {
        log_error("failed to parse device status");
        return -1;
    }
    if (strcmp(status->valuestring, "disable") == 0)
        return 0;
    if (strcmp(status->valuestring, "enable") != 0) {
        log_error("unknown device status %s", status->valuestring);
        return -1;
    }
    cfg->enabled = true;

    cJSON *type = SAFE_CJSON_GET_OBJECT_ITEM(device_json, "type");
    if (!cJSON_IsString(type) ||
        parse_virtio_device_type(type->valuestring, &cfg->type) != 0)
        return -1;

    if (parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "addr"),
                       &cfg->addr) != 0 ||
        parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "len"),
                       &cfg->len) != 0 ||
        parse_json_u32(SAFE_CJSON_GET_OBJECT_ITEM(device_json, "irq"),
                       &cfg->irq) != 0 ||
        cfg->addr == 0 || cfg->len == 0 || cfg->irq == 0) {
        log_error("failed to parse addr, len, or irq");
        return -1;
    }

    cfg->config_ops = virtio_lookup_config_ops(cfg->type);
    if (!cfg->config_ops || !cfg->config_ops->parse ||
        cfg->config_ops->parse(device_json, &cfg->params) != 0) {
        log_error("failed to parse parameters for %s",
                  virtio_device_type_to_string(cfg->type));
        free_virtio_device_config(cfg);
        return -1;
    }
    return 0;
}

static int parse_virtio_memory_region_config(const cJSON *mem_json,
                                             VirtioMemoryRegionConfig *cfg) {
    return parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "zone0_ipa"),
                          &cfg->zone0_ipa) != 0 ||
                   parse_json_u64(
                       SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "zonex_ipa"),
                       &cfg->zonex_ipa) != 0 ||
                   parse_json_u64(SAFE_CJSON_GET_OBJECT_ITEM(mem_json, "size"),
                                  &cfg->size) != 0
               ? -1
               : 0;
}

static int parse_virtio_zone_config(const cJSON *zone_json,
                                    VirtioZoneConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cJSON *id = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "id");
    if (!id)
        id = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "zone_id");
    if (parse_json_u32(id, &cfg->zone_id) != 0 || cfg->zone_id >= MAX_ZONES)
        return -1;

    cJSON *memory = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "memory_region");
    int memory_num = SAFE_CJSON_GET_ARRAY_SIZE(memory);
    if (memory_num < 0 || memory_num > CONFIG_MAX_MEMORY_REGIONS)
        return -1;
    cfg->memory_region_num = (size_t)memory_num;
    for (size_t i = 0; i < cfg->memory_region_num; i++)
        if (parse_virtio_memory_region_config(
                SAFE_CJSON_GET_ARRAY_ITEM(memory, i),
                &cfg->memory_regions[i]) != 0)
            return -1;

    cJSON *devices = SAFE_CJSON_GET_OBJECT_ITEM(zone_json, "devices");
    int device_num = SAFE_CJSON_GET_ARRAY_SIZE(devices);
    if (device_num < 0 || device_num > MAX_DEVS)
        return -1;
    cfg->device_num = (size_t)device_num;
    for (size_t i = 0; i < cfg->device_num; i++)
        if (parse_virtio_device_config(SAFE_CJSON_GET_ARRAY_ITEM(devices, i),
                                       cfg->zone_id, &cfg->devices[i]) != 0)
            return -1;
    return 0;
}

static int parse_virtio_config(const cJSON *root, VirtioConfig *cfg) {
    memset(cfg, 0, sizeof(*cfg));
    cJSON *zones = cJSON_GetObjectItemCaseSensitive(root, "zones");
    if (!zones) {
        cfg->zone_num = 1;
        return parse_virtio_zone_config(root, &cfg->zones[0]);
    }

    int zone_num = SAFE_CJSON_GET_ARRAY_SIZE(zones);
    if (zone_num < 0 || zone_num > MAX_ZONES)
        return -1;
    cfg->zone_num = (size_t)zone_num;
    for (size_t i = 0; i < cfg->zone_num; i++)
        if (parse_virtio_zone_config(SAFE_CJSON_GET_ARRAY_ITEM(zones, i),
                                     &cfg->zones[i]) != 0)
            return -1;
    return 0;
}

static uint64_t range_end(uint64_t base, uint64_t len) {
    return UINT64_MAX - base < len ? UINT64_MAX : base + len;
}

static bool ranges_overlap(uint64_t base_a, uint64_t len_a, uint64_t base_b,
                           uint64_t len_b) {
    if (len_a == 0 || len_b == 0)
        return false;
    return base_a < range_end(base_b, len_b) &&
           base_b < range_end(base_a, len_a);
}

int virtio_validate_config_devices(const VirtioConfig *cfg) {
    size_t total = 0;
    pthread_mutex_lock(&VDEV_MUTEX);
    total = (size_t)vdevs_num;
    for (size_t zi = 0; zi < cfg->zone_num; zi++) {
        const VirtioZoneConfig *zone = &cfg->zones[zi];
        for (size_t di = 0; di < zone->device_num; di++) {
            const VirtioDeviceConfig *dev = &zone->devices[di];
            if (!dev->enabled)
                continue;
            if (++total > MAX_DEVS)
                goto fail;
            for (int i = 0; i < vdevs_num; i++)
                if (vdevs[i]->zone_id == dev->zone_id &&
                    ranges_overlap(dev->addr, dev->len, vdevs[i]->base_addr,
                                   vdevs[i]->len))
                    goto fail;
            for (size_t pzi = 0; pzi <= zi; pzi++) {
                const VirtioZoneConfig *previous = &cfg->zones[pzi];
                size_t limit = pzi == zi ? di : previous->device_num;
                for (size_t pdi = 0; pdi < limit; pdi++) {
                    const VirtioDeviceConfig *previous_dev =
                        &previous->devices[pdi];
                    if (previous_dev->enabled &&
                        previous_dev->zone_id == dev->zone_id &&
                        ranges_overlap(dev->addr, dev->len, previous_dev->addr,
                                       previous_dev->len))
                        goto fail;
                }
            }
        }
    }
    pthread_mutex_unlock(&VDEV_MUTEX);
    return 0;
fail:
    pthread_mutex_unlock(&VDEV_MUTEX);
    log_error("virtio device limit or MMIO range conflict");
    return -1;
}

static int create_virtio_config_devices(const VirtioConfig *cfg) {
    VirtIODevice *new_devs[MAX_DEVS] = {0};
    size_t count = 0;
    for (size_t zi = 0; zi < cfg->zone_num; zi++) {
        const VirtioZoneConfig *zone = &cfg->zones[zi];
        for (size_t di = 0; di < zone->device_num; di++) {
            const VirtioDeviceConfig *dev = &zone->devices[di];
            if (!dev->enabled)
                continue;
            VirtIODevice *vdev = create_virtio_device_unpublished(
                dev->type, dev->zone_id, dev->addr, dev->len, dev->irq,
                &dev->params);
            if (!vdev || count == MAX_DEVS) {
                if (vdev)
                    destroy_unpublished_virtio_device(vdev);
                for (size_t i = 0; i < count; i++)
                    destroy_unpublished_virtio_device(new_devs[i]);
                return -1;
            }
            new_devs[count++] = vdev;
        }
    }
    if (publish_virtio_devices(new_devs, count) != 0) {
        for (size_t i = 0; i < count; i++)
            destroy_unpublished_virtio_device(new_devs[i]);
        return -1;
    }
    return 0;
}

int virtio_start_from_json(char *json_path) {
    uint64_t file_size = 0;
    char *buffer = read_file(json_path, &file_size);
    if (!buffer)
        return -1;
    buffer[file_size] = '\0';
    cJSON *root = SAFE_CJSON_PARSE(buffer);
    VirtioConfig config;
    int err = -1;
    if (!root)
        goto out;
    if (parse_virtio_config(root, &config) != 0 ||
        virtio_validate_config_devices(&config) != 0)
        goto out_config;
    for (size_t i = 0; i < config.zone_num; i++)
        if (virtio_mmap_zone_memory(&config.zones[i]) != 0)
            goto out_config;
    if (create_virtio_config_devices(&config) != 0)
        goto out_config;
    err = 0;
out_config:
    free_virtio_config(&config);
    cJSON_Delete(root);
out:
    free(buffer);
    return err;
}
