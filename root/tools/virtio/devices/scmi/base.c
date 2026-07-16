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

/* SCMI version 2.1 */
#define SCMI_BASE_VERSION 0x20001

/* Helper to get protocol list — per-device table first, then global */
static int get_protocol_list(SCMIDev *dev,
                             struct scmi_msg_resp_base_protocol_list *resp,
                             int skip, int max) {
    int total = dev->protocol_count;
    int remaining = total - skip;

    if (remaining <= 0) {
        resp->count = 0;
        return 0;
    }

    int count = (remaining < max) ? remaining : max;
    resp->count = count;

    for (int i = 0; i < count; i++) {
        uint8_t pid = dev->protocols[skip + i].protocol_id;
        log_info("Protocol %d: %u", skip + i, pid);
        if (i % 4 == 0)
            resp->protocol_slots[i / 4] = 0;
        resp->protocol_slots[i / 4] |= (uint32_t)pid << ((i % 4) * 8);
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
    if (!attr) {
        log_error("handle_base_attributes: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    attr->num_protocols = dev->protocol_count;
    attr->num_agents = 1;
    attr->reserved = 0;

    log_debug("PROTOCOL_ATTRIBUTES resp: num_protocols=%d",
              attr->num_protocols);
    return 0;
}

static int handle_base_version(SCMIDev *dev, uint16_t token,
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

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE, SCMI_COMMON_MSG_VERSION, token,
                       SCMI_SUCCESS);
    uint32_t *version = scmi_resp_write(ctx, sizeof(uint32_t));
    if (!version) {
        log_error("handle_base_version: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    *version = SCMI_BASE_VERSION;
    return 0;
}

static int handle_base_vendor(SCMIDev *dev, uint16_t token,
                              const struct iovec *req_iov,
                              struct scmi_resp_ctx *ctx, bool sub_vendor) {
    (void)dev;
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
    if (!vendor_id) {
        log_error("handle_base_vendor: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
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

    /* Allocate payload via scmi_resp_write using the struct */
    struct scmi_msg_resp_base_protocol_list *resp =
        scmi_resp_write(ctx, sizeof(*resp));
    if (!resp) {
        log_error("handle_base_protocol_list: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }

    int total = dev->protocol_count;
    int max_protos = total > skip ? total - skip : 0;
    int count = get_protocol_list(dev, resp, skip, max_protos);
    /* Adjust to actual bytes written */
    size_t actual = sizeof(uint32_t) + ((count + 3) / 4) * sizeof(uint32_t);
    ctx->written = sizeof(struct scmi_response) + actual;
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
    (void)dev;
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        ctx->capacity, sizeof(struct scmi_response) + 16);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid agent discovery request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t agent_id = *(uint32_t *)req->payload;

    if (agent_id != 0xFFFFFFFF && agent_id != 0) {
        log_error("Agent not found: %u", agent_id);
        return scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                                  SCMI_BASE_MSG_DISCOVER_AGENT, token,
                                  SCMI_ERR_ENTRY);
    }

    scmi_make_response(ctx, SCMI_PROTO_ID_BASE, SCMI_BASE_MSG_DISCOVER_AGENT,
                       token, SCMI_SUCCESS);
    char *name = scmi_resp_write(ctx, 16);
    if (!name) {
        log_error("handle_base_discover_agent: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }

    if (agent_id == 0xFFFFFFFF)
        strncpy(name, "OSPM", 16);
    else
        strncpy(name, "platform", 16);
    return 0;
}

static int handle_base_impl_version(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct scmi_resp_ctx *ctx) {
    (void)dev;
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
    if (!impl_ver) {
        log_error("handle_base_impl_version: scmi_resp_write failed");
        return SCMI_ERR_PARAMS;
    }
    *impl_ver = 0x1;
    return 0;
}

/* Error Notification Handler */
static int handle_base_error_notify(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct scmi_resp_ctx *ctx) {
    (void)dev;
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        ctx->capacity, 0);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid error notification request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t event_control = *(uint32_t *)req->payload;
    log_debug("Error notification %s",
              (event_control & BASE_TP_NOTIFY_ALL) ? "enabled" : "disabled");
    return scmi_make_response(ctx, SCMI_PROTO_ID_BASE,
                              SCMI_BASE_MSG_NOTIFY_ERRORS, token, SCMI_SUCCESS);
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
