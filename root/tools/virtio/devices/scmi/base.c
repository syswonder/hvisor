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
#include "log.h"
#include "virtio_scmi.h"
#include <stdint.h>
#include <string.h>

/* Base protocol handlers are stateless - dev and token are unused */
#pragma GCC diagnostic ignored "-Wunused-parameter"

/* SCMI version 2.1 */
#define SCMI_BASE_VERSION 0x20001

/* Helper to get protocol list from global registration table */
static int get_protocol_list(uint32_t *buffer, int skip, int max) {
    int total = scmi_get_protocol_count();
    int remaining = total - skip;

    if (remaining <= 0) {
        buffer[0] = 0;
        return 0;
    }

    int count = (remaining < max) ? remaining : max;
    buffer[0] = count;

    for (int i = 0; i < count; i++) {
        const struct scmi_protocol *p = scmi_get_protocol_by_index(skip + i);
        uint8_t pid = p ? p->id : 0;
        log_info("Protocol %d: %u", skip + i, pid);
        if (i % 4 == 0)
            buffer[i / 4 + 1] = 0;
        buffer[i / 4 + 1] |= pid << ((i % 4) * 8);
    }
    return count;
}

/* Base Protocol Handlers */
static int handle_base_attributes(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) +
            sizeof(struct scmi_msg_resp_base_attributes));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                       SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES, token,
                       SCMI_SUCCESS);

    struct scmi_msg_resp_base_attributes *attr =
        scmi_resp_write(ctx, sizeof(struct scmi_msg_resp_base_attributes));
    attr->num_protocols = scmi_get_protocol_count();
    attr->num_agents = 1;
    attr->reserved = 0;

    log_debug("PROTOCOL_ATTRIBUTES resp: num_protocols=%d",
              attr->num_protocols);
    return 0;
}

static int handle_base_version(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov,
                               struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid version request");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE, SCMI_COMMON_MSG_VERSION, token,
                       SCMI_SUCCESS);
    uint32_t *version = scmi_resp_write(ctx, sizeof(uint32_t));
    *version = SCMI_BASE_VERSION;
    return 0;
}

static int handle_base_vendor(SCMIDev *dev, uint16_t token,
                              const struct iovec *req_iov,
                              struct scmi_resp_ctx *ctx, bool sub_vendor) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + SCMI_BASE_VENDOR_ID_LEN);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid vendor request");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                       sub_vendor ? SCMI_BASE_MSG_DISCOVER_SUB_VENDOR
                                  : SCMI_BASE_MSG_DISCOVER_VENDOR,
                       token, SCMI_SUCCESS);
    char *vendor_id = scmi_resp_write(ctx, SCMI_BASE_VENDOR_ID_LEN);
    strncpy(vendor_id, sub_vendor ? "SUB_HVIS" : "HVIS",
            SCMI_BASE_VENDOR_ID_LEN);
    return 0;
}

static int handle_base_protocol_list(SCMIDev *dev, uint16_t token,
                                     const struct iovec *req_iov,
                                     struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        ctx->capacity, sizeof(struct scmi_response) + 2 * sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid protocol list request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t skip = *(uint32_t *)req->payload;

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                       SCMI_BASE_MSG_DISCOVER_LIST_PROTOCOLS, token,
                       SCMI_SUCCESS);

    size_t remaining = ctx->capacity - ctx->written;
    uint32_t *protocols =
        (uint32_t *)((uint8_t *)ctx->iov->iov_base + ctx->written);
    size_t max_protos = remaining / sizeof(uint32_t);
    int count = get_protocol_list(protocols, skip, max_protos);
    ctx->written += sizeof(uint32_t) + ((count + 3) / 4) * sizeof(uint32_t);
    if (count < 0) {
        log_error("Invalid skip value: %u", skip);
    } else {
        log_debug("Returning %d protocols (skip=%u)", count, skip);
    }
    return 0;
}

/* Agent Discovery Handler */
static int handle_base_discover_agent(SCMIDev *dev, uint16_t token,
                                      const struct iovec *req_iov,
                                      struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        ctx->capacity, sizeof(struct scmi_response) + 16);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid agent discovery request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t agent_id = *(uint32_t *)req->payload;

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE, SCMI_BASE_MSG_DISCOVER_AGENT,
                       token, SCMI_SUCCESS);
    char *name = scmi_resp_write(ctx, 16);
    memset(name, 0, 16);

    if (agent_id == 0xFFFFFFFF) {
        strncpy(name, "OSPM", 16);
    } else if (agent_id == 0) {
        strncpy(name, "platform", 16);
    } else {
        ctx->written = sizeof(struct scmi_response);
        scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                           SCMI_BASE_MSG_DISCOVER_AGENT, token, SCMI_ERR_ENTRY);
        log_error("Agent not found: %u", agent_id);
    }
    return 0;
}

static int handle_base_impl_version(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), ctx->capacity,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid implementation version request");
        return ret;
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                       SCMI_BASE_MSG_DISCOVER_IMPL_VERSION, token,
                       SCMI_SUCCESS);
    uint32_t *impl_ver = scmi_resp_write(ctx, sizeof(uint32_t));
    *impl_ver = 0x1;
    return 0;
}

/* Error Notification Handler */
static int handle_base_error_notify(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct scmi_resp_ctx *ctx) {
    int ret = scmi_validate_request(req_iov->iov_len, sizeof(uint32_t),
                                    ctx->capacity, 0);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid error notification request");
        return ret;
    }

    uint32_t event_control = *(uint32_t *)req_iov->iov_base;
    log_debug("Error notification %s",
              (event_control & BASE_TP_NOTIFY_ALL) ? "enabled" : "disabled");
    return 0;
}

/* Base Protocol Request Dispatcher */
int virtio_scmi_base_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                const struct iovec *req_iov,
                                struct scmi_resp_ctx *ctx) {
    if (ctx->capacity < sizeof(struct scmi_response) ||
        ctx->iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
    case SCMI_COMMON_MSG_VERSION:
        return handle_base_version(dev, token, req_iov, ctx);
    case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
        return handle_base_attributes(dev, token, req_iov, ctx);
    case SCMI_BASE_MSG_NOTIFY_ERRORS:
        return handle_base_error_notify(dev, token, req_iov, ctx);
    case SCMI_BASE_MSG_DISCOVER_VENDOR:
        return handle_base_vendor(dev, token, req_iov, ctx, false);
    case SCMI_BASE_MSG_DISCOVER_SUB_VENDOR:
        return handle_base_vendor(dev, token, req_iov, ctx, true);
    case SCMI_BASE_MSG_DISCOVER_IMPL_VERSION:
        return handle_base_impl_version(dev, token, req_iov, ctx);
    case SCMI_BASE_MSG_DISCOVER_LIST_PROTOCOLS:
        return handle_base_protocol_list(dev, token, req_iov, ctx);
    case SCMI_BASE_MSG_DISCOVER_AGENT:
        return handle_base_discover_agent(dev, token, req_iov, ctx);
    default:
        log_warn("Unsupported Base protocol message: 0x%x", msg_id);
        return SCMI_ERR_SUPPORT;
    }
}

static const struct scmi_protocol base_protocol = {
    .id = SCMI_PROTO_ID_BASE,
    .handle_request = virtio_scmi_base_handle_req,
};

int virtio_scmi_base_init(void) {
    return scmi_register_protocol(&base_protocol);
}
