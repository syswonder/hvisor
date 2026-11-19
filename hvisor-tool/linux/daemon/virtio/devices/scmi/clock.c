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

#include "hvisor.h"
#include "log.h"
#include "virtio_scmi.h"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Clock Protocol version 3.0 */
#define SCMI_CLOCK_VERSION 0x30000

/* SCMI Clock Subcommands */
#define HVISOR_SCMI_CLOCK_DESCRIBE_RATES 0x03
#define HVISOR_SCMI_CLOCK_RATE_GET 0x04
#define HVISOR_SCMI_CLOCK_RATE_SET 0x05
#define HVISOR_SCMI_CLOCK_CONFIG_GET 0x06
#define HVISOR_SCMI_CLOCK_CONFIG_SET 0x07
#define HVISOR_SCMI_CLOCK_NAME_GET 0x08

/* Define additional structures for clock operations */
struct clock_rates_info {
    __u32 clock_id;
    __u32 rate_index;
    __u32 num_rates;
    __u32 remaining;
    __u64 rates[8];
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

/* SCMI Clock Protocol request payload structs */
struct scmi_msg_req_clock_attributes {
    uint32_t clock_id;
} __attribute__((packed));

struct scmi_msg_req_describe_rates {
    uint32_t clock_id;
    uint32_t rate_index;
} __attribute__((packed));

struct scmi_msg_req_rate_set {
    uint32_t flags;
    uint32_t clock_id;
    uint32_t rate_low;
    uint32_t rate_high;
} __attribute__((packed));

struct scmi_msg_req_rate_get {
    uint32_t clock_id;
} __attribute__((packed));

struct scmi_msg_req_config_set {
    uint32_t clock_id;
    uint32_t attributes;
} __attribute__((packed));

struct scmi_msg_req_config_get {
    uint32_t clock_id;
    uint32_t flags;
} __attribute__((packed));

struct scmi_msg_req_name_get {
    uint32_t clock_id;
} __attribute__((packed));

/* Response for CLOCK_ATTRIBUTES (Message ID 0x3) */
struct scmi_msg_resp_clock_clock_attributes {
    uint32_t attributes;
    char clock_name[16];
} __attribute__((packed));

/* Helper: validate clock_id and get physical ID */
static uint32_t clk_phys_id(SCMIDev *dev, uint32_t clk_id, bool *valid) {
    if (!dev->clock_ids) {
        *valid = false;
        return 0;
    }
    if (clk_id >= dev->clock_count) {
        *valid = false;
        return 0;
    }
    *valid = true;
    return dev->clock_ids[clk_id];
}

/* ================ Handlers ================ */

static int handle_clock_version(SCMIDev *dev, uint16_t token,
                                const struct iovec *req_iov,
                                struct scmi_resp_ctx *ctx) {
    (void)dev;
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid version request");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_COMMON_MSG_VERSION, token,
                       SCMI_SUCCESS);
    uint32_t *version = scmi_resp_write(ctx, sizeof(uint32_t));
    if (!version) {
        log_error("handle_clock_version: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    *version = SCMI_CLOCK_VERSION;
    return 0;
}

static int handle_clock_protocol_attributes(SCMIDev *dev, uint16_t token,
                                            const struct iovec *req_iov,
                                            struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) +
            sizeof(struct scmi_msg_resp_clock_attributes));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                       SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES, token,
                       SCMI_SUCCESS);

    struct scmi_msg_resp_clock_attributes *attr =
        scmi_resp_write(ctx, sizeof(struct scmi_msg_resp_clock_attributes));
    attr->num_clocks = dev->clock_count;
    attr->max_async_req = 1;
    attr->reserved = 0;

    log_debug("CLOCK_PROTOCOL_ATTRIBUTES: num_clocks=%d", attr->num_clocks);
    return 0;
}

static int handle_clock_clock_attributes(SCMIDev *dev, uint16_t token,
                                         const struct iovec *req_iov,
                                         struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len < sizeof(struct scmi_request) +
                               sizeof(struct scmi_msg_req_clock_attributes))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id =
        ((struct scmi_msg_req_clock_attributes *)req->payload)->clock_id;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_clock_attributes: invalid clock_id=%u",
                 clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CLK_ATTRIBUTES, token,
                                  SCMI_ERR_ENTRY);
    }

    size_t expected_resp_size =
        sizeof(struct scmi_response) +
        sizeof(struct scmi_msg_resp_clock_clock_attributes);
    if (ctx->capacity < expected_resp_size) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CLK_ATTRIBUTES, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_clock_args args;
    args.u.clock_attr.clock_id = phys_id;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_GET_ATTRIBUTES, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CLK_ATTRIBUTES, token,
                                  SCMI_ERR_GENERIC);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_CLK_ATTRIBUTES,
                       token, SCMI_SUCCESS);

    struct scmi_msg_resp_clock_clock_attributes *attr = scmi_resp_write(
        ctx, sizeof(struct scmi_msg_resp_clock_clock_attributes));

    uint32_t attributes = 0;
    if (args.u.clock_attr.enabled)
        attributes |= (1U << 0);
    if (args.u.clock_attr.parent_id != (uint32_t)(-1))
        attributes |= (1U << 28);

    attr->attributes = attributes;
    strncpy(attr->clock_name, args.u.clock_attr.clock_name, 15);
    attr->clock_name[15] = '\0';

    return 0;
}

static int handle_clock_describe_rates(SCMIDev *dev, uint16_t token,
                                       const struct iovec *req_iov,
                                       struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len < sizeof(struct scmi_request) +
                               sizeof(struct scmi_msg_req_describe_rates))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_describe_rates *r = (void *)req->payload;
    bool valid;

    clk_phys_id(dev, r->clock_id, &valid);
    if (!valid) {
        log_warn("handle_clock_describe_rates: invalid clock_id=%u",
                 r->clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_DESCRIBE_RATES, token,
                                  SCMI_ERR_ENTRY);
    }

    size_t resp_size =
        sizeof(struct scmi_response) + sizeof(uint32_t) + sizeof(uint64_t) * 3;
    if (ctx->capacity < resp_size) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_DESCRIBE_RATES, token,
                                  SCMI_ERR_RANGE);
    }

    if (r->rate_index > 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_DESCRIBE_RATES, token,
                                  SCMI_ERR_RANGE);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_DESCRIBE_RATES,
                       token, SCMI_SUCCESS);

    uint32_t *payload =
        scmi_resp_write(ctx, sizeof(uint32_t) + sizeof(uint64_t) * 3);
    uint32_t num_rates_flags = (1 << 12) | (3 & 0xFFFU);
    payload[0] = num_rates_flags;

    uint64_t *rates = (uint64_t *)&payload[1];
    rates[0] = 0;
    rates[1] = 10000000000ULL;
    rates[2] = 1;

    return 0;
}

static int handle_clock_rate_get(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_rate_get))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id =
        ((struct scmi_msg_req_rate_get *)req->payload)->clock_id;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_rate_get: invalid clock_id=%u", clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_GET, token,
                                  SCMI_ERR_ENTRY);
    }

    if (ctx->capacity < sizeof(struct scmi_response) + 8) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_GET, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_clock_args args;
    struct clock_rate_info *rate_info = (struct clock_rate_info *)&args.u.data;
    rate_info->clock_id = phys_id;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_RATE_GET, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_GET, token,
                                  SCMI_ERR_GENERIC);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_RATE_GET, token,
                       SCMI_SUCCESS);

    uint32_t *payload = scmi_resp_write(ctx, sizeof(uint64_t));
    if (!payload) {
        log_error("handle_clock_rate_get: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    payload[0] = (uint32_t)(rate_info->rate & 0xFFFFFFFFULL);
    payload[1] = (uint32_t)((rate_info->rate >> 32) & 0xFFFFFFFFULL);

    return 0;
}

static int handle_clock_rate_set(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_rate_set))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_rate_set *r = (void *)req->payload;
    uint32_t flags = r->flags;
    uint32_t clock_id = r->clock_id;
    uint64_t requested_rate = ((uint64_t)r->rate_high << 32) | r->rate_low;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_rate_set: invalid clock_id=%u", clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_SET, token,
                                  SCMI_ERR_ENTRY);
    }

    bool async = (flags & 0x1) != 0;
    if (async) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_SET, token,
                                  SCMI_ERR_SUPPORT);
    }

    struct hvisor_scmi_clock_args args;
    struct clock_rate_set_info *rate_set_info =
        (struct clock_rate_set_info *)&args.u.data;
    rate_set_info->clock_id = phys_id;
    rate_set_info->flags = flags;
    rate_set_info->rate = requested_rate;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_RATE_SET, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_RATE_SET, token,
                                  SCMI_ERR_GENERIC);
    }

    return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_RATE_SET,
                              token, SCMI_SUCCESS);
}

static int handle_clock_config_get(SCMIDev *dev, uint16_t token,
                                   const struct iovec *req_iov,
                                   struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_config_get))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_config_get *r = (void *)req->payload;
    uint32_t clock_id = r->clock_id;
    uint32_t flags = r->flags;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_config_get: invalid clock_id=%u", clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_GET, token,
                                  SCMI_ERR_ENTRY);
    }

    uint32_t ext_type = flags & 0xFF;
    if (ext_type != 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_GET, token,
                                  SCMI_ERR_PARAMS);
    }

    if (ctx->capacity < sizeof(struct scmi_response) + 12) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_GET, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_clock_args args;
    struct clock_config_info *config_info =
        (struct clock_config_info *)&args.u.data;
    config_info->clock_id = phys_id;
    config_info->flags = flags;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_CONFIG_GET, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_GET, token,
                                  SCMI_ERR_GENERIC);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_CONFIG_GET,
                       token, SCMI_SUCCESS);

    uint32_t *payload = scmi_resp_write(ctx, sizeof(uint32_t) * 3);
    if (!payload) {
        log_error("handle_clock_config_get: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    payload[0] = 0;
    payload[1] = config_info->config;
    payload[2] = config_info->extended_config_val;

    return 0;
}

static int handle_clock_config_set(SCMIDev *dev, uint16_t token,
                                   const struct iovec *req_iov,
                                   struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_config_set))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_config_set *r = (void *)req->payload;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, r->clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_config_set: invalid clock_id=%u", r->clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_SET, token,
                                  SCMI_ERR_ENTRY);
    }

    struct hvisor_scmi_clock_args args;
    struct clock_config_info *config_info =
        (struct clock_config_info *)&args.u.data;
    config_info->clock_id = phys_id;
    config_info->config = r->attributes;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_CONFIG_SET, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_CONFIG_SET, token,
                                  SCMI_ERR_GENERIC);
    }

    return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                              SCMI_CLOCK_MSG_CONFIG_SET, token, SCMI_SUCCESS);
}

static int handle_clock_name_get(SCMIDev *dev, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_name_get))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    uint32_t clock_id =
        ((struct scmi_msg_req_name_get *)req->payload)->clock_id;
    bool valid;
    uint32_t phys_id = clk_phys_id(dev, clock_id, &valid);

    if (!valid) {
        log_warn("handle_clock_name_get: invalid clock_id=%u", clock_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_NAME_GET, token,
                                  SCMI_ERR_ENTRY);
    }

    size_t payload_size = 4 + 64;
    if (ctx->capacity < sizeof(struct scmi_response) + payload_size) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_NAME_GET, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_clock_args args;
    struct clock_name_info *name_info = (struct clock_name_info *)&args.u.data;
    name_info->clock_id = phys_id;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_CLOCK_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_CLOCK_NAME_GET, "clock");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK,
                                  SCMI_CLOCK_MSG_NAME_GET, token,
                                  SCMI_ERR_GENERIC);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_CLOCK, SCMI_CLOCK_MSG_NAME_GET, token,
                       SCMI_SUCCESS);

    uint32_t *flags = scmi_resp_write(ctx, 4 + 64);
    char *name = (char *)(flags + 1);
    *flags = 0;
    strncpy(name, name_info->name, 63);
    name[63] = '\0';

    return 0;
}

/* Main request handler */
int virtio_scmi_clock_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx) {
    if (ctx->capacity < sizeof(struct scmi_response) ||
        ctx->iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
    case SCMI_COMMON_MSG_VERSION:
        return handle_clock_version(dev, token, req_iov, ctx);
    case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
        return handle_clock_protocol_attributes(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_CLK_ATTRIBUTES:
        return handle_clock_clock_attributes(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_DESCRIBE_RATES:
        return handle_clock_describe_rates(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_RATE_SET:
        return handle_clock_rate_set(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_RATE_GET:
        return handle_clock_rate_get(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_CONFIG_SET:
        return handle_clock_config_set(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_CONFIG_GET:
        return handle_clock_config_get(dev, token, req_iov, ctx);
    case SCMI_CLOCK_MSG_NAME_GET:
        return handle_clock_name_get(dev, token, req_iov, ctx);
    default:
        log_warn("Unsupported Clock protocol message: 0x%x", msg_id);
        return SCMI_ERR_SUPPORT;
    }
}
