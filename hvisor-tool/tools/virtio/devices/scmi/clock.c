// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Linkun Chen <lkchen01@foxmail.com>
 */

#include "virtio_scmi.h"
#include "log.h"
#include "hvisor.h"
#include "safe_cjson.h"
#include <string.h>
#include <stdint.h>
#include <sys/ioctl.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <time.h>
#include <stdlib.h>

// Clock map context
static scmi_map_context_t clock_map_ctx = {
    .map = NULL,
    .map_count = 0,
    .allowed_ids = NULL,
    .allowed_count = 0,
    .allow_all = false
};

/* Clock Protocol version 3.0 */
#define SCMI_CLOCK_VERSION 0x30000

/* SCMI Clock Subcommands */
#define HVISOR_SCMI_CLOCK_DESCRIBE_RATES 0x03
#define HVISOR_SCMI_CLOCK_RATE_GET       0x04
#define HVISOR_SCMI_CLOCK_RATE_SET       0x05
#define HVISOR_SCMI_CLOCK_CONFIG_GET     0x06
#define HVISOR_SCMI_CLOCK_CONFIG_SET     0x07
#define HVISOR_SCMI_CLOCK_NAME_GET       0x08

/* Define additional structures for clock operations */
struct clock_rates_info {
    __u32 clock_id;
    __u32 rate_index;
    __u32 num_rates;
    __u32 remaining;
    __u64 rates[8]; /* Max 8 rates per response */
};

struct clock_rate_info {
    __u32 clock_id;
    __u64 rate;
};

struct clock_rate_set_info {
    __u32 clock_id;
    __u32 flags;
    __u64 rate;
};

struct clock_config_info {
    __u32 clock_id;
    __u32 flags;
    __u32 config;
    __u32 extended_config_val;
};

struct clock_name_info {
    __u32 clock_id;
    char name[64];
};

/* Forward declarations */
static int scmi_clock_get_count(uint16_t *clock_count);

/* Cache for clock count */
static uint16_t cached_clock_count = 0;
static bool clock_count_cache_valid = false;

/* Response for CLOCK_ATTRIBUTES (Message ID 0x3) */
struct scmi_msg_resp_clock_clock_attributes {
    uint32_t attributes;          /* Bitfield as per spec */
    char clock_name[16];          /* Null-terminated, max 15 chars + \0 */
    // uint32_t clock_enable_delay;  /* in microseconds */
};



/* Helper: validate clock_id */
static bool is_valid_clock_id(uint32_t clock_id) {
    if (clock_map_ctx.allow_all) {
        uint16_t total_clocks = 0;
        if (scmi_clock_get_count(&total_clocks) < 0) {
            return false;
        }
        return clock_id < total_clocks;
    }
    return scmi_is_valid_id(&clock_map_ctx, clock_id);
}

/* ================ Handlers ================ */

static int handle_clock_version(SCMIDev *dev, uint16_t token,
                             const struct iovec *req_iov,
                             struct iovec *resp_iov) {
    int ret = scmi_validate_request(req_iov->iov_len, sizeof(struct scmi_request),
                                  resp_iov->iov_len, sizeof(struct scmi_response) +
                                  sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid version request");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *version = (uint32_t *)resp->payload;
    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    *version = SCMI_CLOCK_VERSION;
    return 0;
}

static int hvisor_scmi_ioctl(uint32_t subcmd, struct hvisor_scmi_clock_args *ioctl_args, size_t args_size) {
    int fd = open(HVISOR_DEVICE, O_RDWR);
    if (fd < 0) {
        log_error("Failed to open hvisor device");
        return -ENODEV;
    }

    ioctl_args->subcmd = subcmd;
    ioctl_args->data_len = args_size - offsetof(struct hvisor_scmi_clock_args, u);

    if (ioctl(fd, HVISOR_SCMI_CLOCK_IOCTL, ioctl_args) < 0) {
        log_error("Failed to perform SCMI clock ioctl, subcmd=%u: %s",
                 subcmd, strerror(errno));
        close(fd);
        return -EIO;
    }
    close(fd);

    return 0;
}

static int scmi_clock_get_count(uint16_t *clock_count) {
    if (clock_count_cache_valid) {
        *clock_count = cached_clock_count;
        return 0;
    }

    struct hvisor_scmi_clock_args args;
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_GET_COUNT, &args, sizeof(args));
    if (ret < 0) {
        return ret;
    }

    *clock_count = args.u.clock_count;
    cached_clock_count = args.u.clock_count;
    clock_count_cache_valid = true;

    return 0;
}

static int handle_clock_protocol_attributes(SCMIDev *dev, uint16_t token,
                                const struct iovec *req_iov,
                                struct iovec *resp_iov) {
    int ret = scmi_validate_request(req_iov->iov_len, sizeof(struct scmi_request),
                                  resp_iov->iov_len, sizeof(struct scmi_response) +
                                  sizeof(struct scmi_msg_resp_clock_attributes));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_clock_attributes *attr = (struct scmi_msg_resp_clock_attributes *)resp->payload;

    // Use clock_max_num from configuration
    extern uint32_t clock_max_num;
    attr->num_clocks = clock_max_num;

    attr->max_async_req = 1; // support 1 async request
    attr->reserved = 0;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);

    log_debug("CLOCK_PROTOCOL_ATTRIBUTES: num_clocks=%d", attr->num_clocks);
    return 0;
}

static int handle_clock_clock_attributes(SCMIDev *dev, uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id = *(uint32_t *)req->payload;

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    // Validate response buffer size
    size_t expected_resp_size = sizeof(struct scmi_response) +
                                sizeof(struct scmi_msg_resp_clock_clock_attributes);
    if (resp_iov->iov_len < expected_resp_size) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    // Prepare ioctl arguments
    struct hvisor_scmi_clock_args args;
    args.u.clock_attr.clock_id = scmi_map_id(&clock_map_ctx, clock_id);

    // Call the common ioctl function
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_GET_ATTRIBUTES, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_clock_clock_attributes *attr =
        (struct scmi_msg_resp_clock_clock_attributes *)resp->payload;

    // Build attributes field
    uint32_t attributes = 0;
    // Bit[0]: enabled
    if (args.u.clock_attr.enabled) {
        attributes |= (1U << 0);
    }
    // Bit[1]: restricted clock? (we don't implement permissions, so assume not restricted)
    // attributes |= (0 << 1); // default 0

    // Bit[27]: extended config support? (we only support basic enable/disable)
    // -> set to 0

    // Bit[28]: parent clock identifier support?
    if (args.u.clock_attr.parent_id != (uint32_t)(-1)) {
        attributes |= (1U << 28);
    }

    // Bit[29]: extended name? (our names are "fake_clk_X", <16 bytes)
    // -> set to 0 (no extended name needed)

    // Bit[30], [31]: notifications (rate change, etc.) – not supported
    // -> leave as 0

    attr->attributes = attributes;

    // Copy clock name (max 15 chars + null)
    strncpy(attr->clock_name, args.u.clock_attr.clock_name, 15);
    attr->clock_name[15] = '\0';

    log_debug("CLOCK_CLOCK_ATTRIBUTES: clock_id=%u, name=%s, enabled=%d, is_valid=%d",
             clock_id, attr->clock_name, args.u.clock_attr.enabled, args.u.clock_attr.is_valid);

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    log_debug("CLOCK_ATTRIBUTES: clock_id=%u, name=%s, enabled=%d",
              clock_id, attr->clock_name, args.u.clock_attr.enabled);
    return 0;
}
static int handle_clock_describe_rates(SCMIDev *dev, uint16_t token,
                                      const struct iovec *req_iov,
                                      struct iovec *resp_iov)
{
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id = *(uint32_t *)req->payload;
    uint32_t rate_index = *(uint32_t *)((uint8_t *)req->payload + 4);

    /* Validate clock ID */
    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    /* Calculate required response buffer size */
    size_t resp_hdr_size = sizeof(struct scmi_response); // includes status (4 bytes)

    if (resp_iov->iov_len < resp_hdr_size + sizeof(uint32_t)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    if (rate_index > 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    // /* Prepare ioctl arguments */
    // struct hvisor_scmi_clock_args args;
    // struct clock_rates_info *rates_info = (struct clock_rates_info *)&args.u.data;
    // rates_info->clock_id = clock_id;
    // rates_info->rate_index = rate_index;

    // /* Call the common ioctl function */
    // int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_DESCRIBE_RATES, &args, sizeof(args));
    // if (ret < 0) {
    //     return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    // }

    /* Prepare response */
    struct scmi_response *resp = (struct scmi_response *)resp_iov->iov_base;
    uint32_t *payload = (uint32_t *)resp->payload;

    /* num_rates_flags:
     *   Bit[12] = 1 (continuous rates)
     *   Bits[31:16] = 0
     *   Bits[11:0] = 3
     */
    uint32_t num_rates_flags = (1 << 12) | (3 & 0xFFFU);
    payload[0] = num_rates_flags;

    /* Calculate how many rates we can fit in the response buffer */
    // size_t available_for_rates = resp_iov->iov_len - resp_hdr_size - fixed_payload_size;
    // uint32_t max_rates_by_buffer = (uint32_t)(available_for_rates / rate_entry_size);
    // uint32_t num_return = (rates_info->num_rates < max_rates_by_buffer) ? rates_info->num_rates : max_rates_by_buffer;

    /* Fill rates: each rate is {low32, high32, step_size} */
    // All rates are available in one response for simplicity
    uint64_t *rates = &payload[1];
    rates[0] = 0;
    rates[1] = 10000000000ULL;     // 10 GHz
    rates[2] = 1;

    log_debug("CLOCK_DESCRIBE_RATES: clock_id=%u, rate_index=%u, num_rates=%u",
              clock_id, rate_index, 3);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_clock_rate_get(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov,
                               struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id = *(uint32_t *)req->payload;

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    if (resp_iov->iov_len < sizeof(struct scmi_response) + 8) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_clock_args args;
    struct clock_rate_info *rate_info = (struct clock_rate_info *)&args.u.data;
    rate_info->clock_id = scmi_map_id(&clock_map_ctx, clock_id);

    /* Call the common ioctl function */
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_RATE_GET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *payload = (uint32_t *)resp->payload;
    payload[0] = (uint32_t)(rate_info->rate & 0xFFFFFFFFULL);
    payload[1] = (uint32_t)((rate_info->rate >> 32) & 0xFFFFFFFFULL);

    log_debug("CLOCK_RATE_GET: clock_id=%u, rate=%llu Hz", clock_id, rate_info->rate);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_clock_rate_set(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov,
                               struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint8_t *p = req->payload;
    uint32_t flags = *(uint32_t *)p; p += 4;
    uint32_t clock_id = *(uint32_t *)p; p += 4;
    uint32_t rate_lo = *(uint32_t *)p; p += 4;
    uint32_t rate_hi = *(uint32_t *)p;
    uint64_t requested_rate = ((uint64_t)rate_hi << 32) | rate_lo;

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    bool async = (flags & 0x1) != 0;
    // For simplicity, we only support synchronous mode
    if (async) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_SUPPORT);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_clock_args args;
    struct clock_rate_set_info *rate_set_info = (struct clock_rate_set_info *)&args.u.data;
    rate_set_info->clock_id = scmi_map_id(&clock_map_ctx, clock_id);
    rate_set_info->flags = flags;
    rate_set_info->rate = requested_rate;

    /* Call the common ioctl function */
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_RATE_SET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    log_debug("CLOCK_RATE_SET: clock_id=%u, flags=%u, rate=%llu Hz", clock_id, flags, requested_rate);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_clock_config_get(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id = *(uint32_t *)req->payload;
    uint32_t flags = *(uint32_t *)((uint8_t *)req->payload + 4);

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    uint32_t ext_type = flags & 0xFF;

    if (ext_type != 0) {
        // Only support basic enable/disable for now
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_PARAMS);
    }

    if (resp_iov->iov_len < sizeof(struct scmi_response) + 12) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_clock_args args;
    struct clock_config_info *config_info = (struct clock_config_info *)&args.u.data;
    config_info->clock_id = scmi_map_id(&clock_map_ctx, clock_id);
    config_info->flags = flags;

    /* Call the common ioctl function */
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_CONFIG_GET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *payload = (uint32_t *)resp->payload;
    payload[0] = 0; // attributes (reserved)
    payload[1] = config_info->config;
    payload[2] = config_info->extended_config_val;

    log_debug("CLOCK_CONFIG_GET: clock_id=%u, flags=%u, config=%u, extended_config_val=%u",
             clock_id, flags, config_info->config, config_info->extended_config_val);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_clock_config_set(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint8_t *p = req->payload;
    uint32_t clock_id = *(uint32_t *)p; p += 4;
    uint32_t attributes = *(uint32_t *)p;

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_clock_args args;
    struct clock_config_info *config_info = (struct clock_config_info *)&args.u.data;
    config_info->clock_id = scmi_map_id(&clock_map_ctx, clock_id);
    config_info->config = attributes;

    /* Call the common ioctl function */
    // struct timespec start, end;
    // clock_gettime(CLOCK_MONOTONIC, &start);

    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_CONFIG_SET, &args, sizeof(args));

    // clock_gettime(CLOCK_MONOTONIC, &end);
    // double elapsed = (end.tv_sec - start.tv_sec) * 1000000.0 + (end.tv_nsec - start.tv_nsec) / 1000.0;

    if (ret < 0) {
        // log_error("CLOCK_CONFIG_SET: clock_id=%u, config=%u, ioctl failed in %.2f us", clock_id, attributes, elapsed);
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    // log_error("CLOCK_CONFIG_SET: clock_id=%u, config=%u, ioctl took %.2f us", clock_id, attributes, elapsed);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_clock_name_get(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov,
                               struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id = *(uint32_t *)req->payload;

    if (!is_valid_clock_id(clock_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    size_t payload_size = 4 + 64; // flags (4) + name[64]

    if (resp_iov->iov_len < sizeof(struct scmi_response) + payload_size) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_clock_args args;
    struct clock_name_info *name_info = (struct clock_name_info *)&args.u.data;
    name_info->clock_id = scmi_map_id(&clock_map_ctx, clock_id);

    /* Call the common ioctl function */
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_CLOCK_NAME_GET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *flags = (uint32_t *)resp->payload;
    char *name = (char *)(flags + 1);

    *flags = 0;
    strncpy(name, name_info->name, 63);
    name[63] = '\0';

    log_debug("CLOCK_NAME_GET: clock_id=%u, name=%s", clock_id, name_info->name);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

/* Main request handler */
static int virtio_scmi_clock_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                     const struct iovec *req_iov, struct iovec *resp_iov) {
    if (resp_iov->iov_len < sizeof(struct scmi_response) || resp_iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
        case SCMI_COMMON_MSG_VERSION:
            return handle_clock_version(dev, token, req_iov, resp_iov);
        case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
            return handle_clock_protocol_attributes(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_CLK_ATTRIBUTES:
            return handle_clock_clock_attributes(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_DESCRIBE_RATES:
            return handle_clock_describe_rates(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_RATE_SET:
            return handle_clock_rate_set(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_RATE_GET:
            return handle_clock_rate_get(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_CONFIG_SET:
            return handle_clock_config_set(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_CONFIG_GET:
            return handle_clock_config_get(dev, token, req_iov, resp_iov);
        case SCMI_CLOCK_MSG_NAME_GET:
            return handle_clock_name_get(dev, token, req_iov, resp_iov);
        default:
            log_warn("Unsupported Clock protocol message: 0x%x", msg_id);
            return SCMI_ERR_SUPPORT;
    }
}

/* Clock Protocol Operations */
static const struct scmi_protocol clock_protocol = {
    .id = SCMI_PROTO_ID_CLOCK,
    .handle_request = virtio_scmi_clock_handle_req,
};

/**
 * virtio_scmi_clock_init_map - Initialize clock allowed list and map from configuration
 * @allowed_list_json: JSON object containing allowed clock IDs
 * @clock_map_json: JSON object containing clock ID mappings
 *
 * Return: 0 on success, negative error code on failure
 */
int virtio_scmi_clock_init_map(cJSON *allowed_list_json, cJSON *clock_map_json) {
    int ret = scmi_init_map(&clock_map_ctx, allowed_list_json, clock_map_json, "clock_ids", "clock_map");
    
    // Set clock provider phandle if available
    extern uint32_t clock_phandle;
    if (clock_phandle > 0) {
        int fd = open(HVISOR_DEVICE, O_RDWR);
        if (fd >= 0) {
            struct hvisor_scmi_clock_args args;
            args.subcmd = HVISOR_SCMI_CLOCK_SET_PHANDLE;
            args.data_len = sizeof(args.u.clock_phandle_info);
            args.u.clock_phandle_info.phandle = clock_phandle;
            if (ioctl(fd, HVISOR_SCMI_CLOCK_IOCTL, &args) < 0) {
                log_error("Failed to set clock provider phandle: %s", strerror(errno));
            } else {
                log_info("Clock provider phandle set to %u", clock_phandle);
            }
            close(fd);
        }
    }
    
    return ret;
}

/* Initialize Clock Protocol */
int virtio_scmi_clock_init(void) {
    return scmi_register_protocol(&clock_protocol);
}