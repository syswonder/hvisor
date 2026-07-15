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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Reset Protocol version 2.0 */
#define SCMI_RESET_VERSION 0x20000

#pragma GCC diagnostic ignored "-Wunused-parameter"

/* Response for RESET_ATTRIBUTES (Message ID 0x3) */
struct scmi_msg_resp_reset_attributes {
    uint32_t attributes;
    uint32_t reset_latency;
    char reset_name[16];
} __attribute__((packed));

/* Request for RESET (Message ID 0x4) */
struct scmi_msg_req_reset {
    uint32_t domain_id;
    uint32_t flags;
    uint32_t reset_state;
} __attribute__((packed));

/* Helper: validate reset_id and get physical ID */
static uint32_t rst_phys_id(SCMIDev *dev, uint32_t rst_id, bool *valid) {
    if (!dev->reset_ids) {
        *valid = false;
        return 0;
    }
    if (rst_id >= dev->reset_count) {
        *valid = false;
        return 0;
    }
    *valid = true;
    return dev->reset_ids[rst_id];
}

static int handle_reset_version(SCMIDev *dev, uint16_t token,
                                const struct iovec *req_iov,
                                struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid version request");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_RESET, SCMI_COMMON_MSG_VERSION, token,
                       SCMI_SUCCESS);
    uint32_t *version = scmi_resp_write(ctx, sizeof(uint32_t));
    *version = SCMI_RESET_VERSION;
    return 0;
}

static int handle_reset_protocol_attributes(SCMIDev *dev, uint16_t token,
                                            const struct iovec *req_iov,
                                            struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                       SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES, token,
                       SCMI_SUCCESS);
    uint32_t *attributes = scmi_resp_write(ctx, sizeof(uint32_t));
    *attributes = (uint32_t)(dev->reset_count & 0xFFFF);

    log_debug("RESET_PROTOCOL_ATTRIBUTES: num_resets=%d", dev->reset_count);
    return 0;
}

static int handle_reset_attributes(SCMIDev *dev, uint16_t token,
                                   const struct iovec *req_iov,
                                   struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len < sizeof(struct scmi_request) + sizeof(uint32_t))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;
    bool valid;
    uint32_t phys_id = rst_phys_id(dev, domain_id, &valid);

    if (!valid) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                                  SCMI_RESET_MSG_RESET_ATTRIBUTES, token,
                                  SCMI_ERR_ENTRY);
    }

    size_t expected_resp_size = sizeof(struct scmi_response) +
                                sizeof(struct scmi_msg_resp_reset_attributes);
    if (ctx->capacity < expected_resp_size) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                                  SCMI_RESET_MSG_RESET_ATTRIBUTES, token,
                                  SCMI_ERR_RANGE);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                       SCMI_RESET_MSG_RESET_ATTRIBUTES, token, SCMI_SUCCESS);
    struct scmi_msg_resp_reset_attributes *attr =
        scmi_resp_write(ctx, sizeof(struct scmi_msg_resp_reset_attributes));
    attr->attributes = 0;
    attr->reset_latency = 100;
    snprintf(attr->reset_name, 15, "rst_%u", phys_id);
    attr->reset_name[15] = '\0';

    log_debug("RESET_RESET_ATTRIBUTES: domain_id=%u, phys_id=%u, name=%s",
              domain_id, phys_id, attr->reset_name);
    return 0;
}

static int handle_reset(SCMIDev *dev, uint16_t token,
                        const struct iovec *req_iov,
                        struct scmi_resp_ctx *ctx) {
    if (req_iov->iov_len <
        sizeof(struct scmi_request) + sizeof(struct scmi_msg_req_reset))
        return SCMI_ERR_PARAMS;
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_reset *reset_req =
        (struct scmi_msg_req_reset *)req->payload;

    uint32_t domain_id = reset_req->domain_id;
    uint32_t flags = reset_req->flags;
    uint32_t reset_state = reset_req->reset_state;
    bool valid;
    uint32_t phys_id = rst_phys_id(dev, domain_id, &valid);

    if (!valid) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                                  SCMI_RESET_MSG_RESET, token, SCMI_ERR_ENTRY);
    }

    bool async = (flags & (1 << 2)) != 0;
    if (async) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                                  SCMI_RESET_MSG_RESET, token,
                                  SCMI_ERR_SUPPORT);
    }

    struct hvisor_scmi_reset_args args;
    args.u.reset_info.domain_id = phys_id;
    args.u.reset_info.flags = flags;
    args.u.reset_info.reset_state = reset_state;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_RESET_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_RESET_RESET, "reset");
    if (ret < 0) {
        return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                                  SCMI_RESET_MSG_RESET, token,
                                  SCMI_ERR_GENERIC);
    }

    log_warn("RESET: domain_id=%u, flags=0x%x, reset_state=%u", domain_id,
             flags, reset_state);

    return scmi_make_response(ctx, SCMI_PROTO_ID_RESET, SCMI_RESET_MSG_RESET,
                              token, SCMI_SUCCESS);
}

static int handle_reset_notify(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov
                               __attribute__((unused)),
                               struct scmi_resp_ctx *ctx) {
    return scmi_make_response(ctx, SCMI_PROTO_ID_RESET,
                              SCMI_RESET_MSG_RESET_NOTIFY, token,
                              SCMI_ERR_SUPPORT);
}

/* Main request handler */
int virtio_scmi_reset_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx) {
    if (ctx->capacity < sizeof(struct scmi_response) ||
        ctx->iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
    case SCMI_COMMON_MSG_VERSION:
        return handle_reset_version(dev, token, req_iov, ctx);
    case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
        return handle_reset_protocol_attributes(dev, token, req_iov, ctx);
    case SCMI_RESET_MSG_RESET_ATTRIBUTES:
        return handle_reset_attributes(dev, token, req_iov, ctx);
    case SCMI_RESET_MSG_RESET:
        return handle_reset(dev, token, req_iov, ctx);
    case SCMI_RESET_MSG_RESET_NOTIFY:
        return handle_reset_notify(dev, token, req_iov, ctx);
    default:
        log_warn("Unsupported Reset protocol message: 0x%x", msg_id);
        return SCMI_ERR_SUPPORT;
    }
}

static const struct scmi_protocol reset_protocol = {
    .id = SCMI_PROTO_ID_RESET,
    .handle_request = virtio_scmi_reset_handle_req,
};

int virtio_scmi_reset_init(void) {
    return scmi_register_protocol(&reset_protocol);
}
