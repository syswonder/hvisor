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
#include <string.h>

/* Protocol Attributes */
#define SCMI_BASE_VENDOR_ID_LEN 16

/* SCMI version 2.1 */
#define SCMI_BASE_VERSION 0x20001

/* Helper to get protocol list */
static int get_protocol_list(uint32_t *buffer, int skip, int max) {
    int num = scmi_get_protocol_count();
    log_info("Total protocols: %d, skip: %d, max: %d", num, skip, max);
    int remaining = num - skip;

    if (remaining <= 0) {
        buffer[0] = 0;
        return 0;
    }

    int count = (remaining < max) ? remaining : max;
    buffer[0] = count;

    for (int i = 0; i < count; i++) {
        log_info("Protocol %d: %u", skip + i,
                 scmi_get_protocol_by_index(skip + i)->id);
        if (i % 4 == 0)
            buffer[i / 4 + 1] = 0;
        buffer[i / 4 + 1] |= scmi_get_protocol_by_index(skip + i)->id
                             << ((i % 4) * 8);
    }
    return count;
}

/* Base Protocol Handlers */
static int handle_base_attributes(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), resp_iov->iov_len,
        sizeof(struct scmi_response) +
            sizeof(struct scmi_msg_resp_base_attributes));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_base_attributes *attr =
        (struct scmi_msg_resp_base_attributes *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    attr->num_protocols = scmi_get_protocol_count();
    attr->num_agents = 1;
    attr->reserved = 0;

    log_debug("PROTOCOL_ATTRIBUTES resp: num_protocols=%d",
              attr->num_protocols);
    return 0;
}

static int handle_base_version(SCMIDev *dev, uint16_t token,
                               const struct iovec *req_iov,
                               struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), resp_iov->iov_len,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid version request");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *version = (uint32_t *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    *version = SCMI_BASE_VERSION;
    return 0;
}

static int handle_base_vendor(SCMIDev *dev, uint16_t token,
                              const struct iovec *req_iov,
                              struct iovec *resp_iov, bool sub_vendor) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), resp_iov->iov_len,
        sizeof(struct scmi_response) + SCMI_BASE_VENDOR_ID_LEN);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid vendor request");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    char *vendor_id = (char *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    strncpy(vendor_id, sub_vendor ? "SUB_HVIS" : "HVIS",
            SCMI_BASE_VENDOR_ID_LEN);
    return 0;
}

static int handle_base_protocol_list(SCMIDev *dev, uint16_t token,
                                     const struct iovec *req_iov,
                                     struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        resp_iov->iov_len, sizeof(struct scmi_response) + 2 * sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid protocol list request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t skip = *(uint32_t *)req->payload;
    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *protocols = (uint32_t *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);

    int count = get_protocol_list(
        protocols, skip, resp_iov->iov_len - sizeof(struct scmi_response));
    if (count < 0) {
        resp->status = -count;
        log_error("Invalid skip value: %u", skip);
    } else {
        log_debug("Returning %d protocols (skip=%u)", count, skip);
    }
    return 0;
}

/* Agent Discovery Handler */
static int handle_base_discover_agent(SCMIDev *dev, uint16_t token,
                                      const struct iovec *req_iov,
                                      struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request) + sizeof(uint32_t),
        resp_iov->iov_len, sizeof(struct scmi_response) + 16);
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid agent discovery request");
        return ret;
    }

    struct scmi_request *req = req_iov->iov_base;
    uint32_t agent_id = *(uint32_t *)req->payload;
    struct scmi_response *resp = resp_iov->iov_base;
    char *name = (char *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    memset(name, 0, 16);

    if (agent_id == 0xFFFFFFFF) {
        strncpy(name, "OSPM", 16);
    } else if (agent_id == 0) {
        strncpy(name, "platform", 16);
    } else {
        resp->status = SCMI_ERR_ENTRY;
        log_error("Agent not found: %u", agent_id);
    }
    return 0;
}

static int handle_base_impl_version(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), resp_iov->iov_len,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid implementation version request");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *impl_ver = (uint32_t *)resp->payload;

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    *impl_ver = 0x1; /* Implementation version 1 */
    return 0;
}

/* Error Notification Handler */
static int handle_base_error_notify(SCMIDev *dev, uint16_t token,
                                    const struct iovec *req_iov,
                                    struct iovec *resp_iov) {
    int ret = scmi_validate_request(req_iov->iov_len, sizeof(uint32_t),
                                    resp_iov->iov_len, 0);
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
static int virtio_scmi_base_handle_req(SCMIDev *dev, uint8_t msg_id,
                                       uint16_t token,
                                       const struct iovec *req_iov,
                                       struct iovec *resp_iov) {
    /* Validate response buffer */
    if (resp_iov->iov_len < sizeof(struct scmi_response) ||
        resp_iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
    case SCMI_COMMON_MSG_VERSION:
        return handle_base_version(dev, token, req_iov, resp_iov);
    case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
        return handle_base_attributes(dev, token, req_iov, resp_iov);
    case SCMI_BASE_MSG_NOTIFY_ERRORS:
        return handle_base_error_notify(dev, token, req_iov, resp_iov);
    case SCMI_BASE_MSG_DISCOVER_VENDOR:
        return handle_base_vendor(dev, token, req_iov, resp_iov, false);
    case SCMI_BASE_MSG_DISCOVER_SUB_VENDOR:
        return handle_base_vendor(dev, token, req_iov, resp_iov, true);
    case SCMI_BASE_MSG_DISCOVER_IMPL_VERSION:
        return handle_base_impl_version(dev, token, req_iov, resp_iov);
    case SCMI_BASE_MSG_DISCOVER_LIST_PROTOCOLS:
        return handle_base_protocol_list(dev, token, req_iov, resp_iov);
    case SCMI_BASE_MSG_DISCOVER_AGENT:
        return handle_base_discover_agent(dev, token, req_iov, resp_iov);
    default:
        log_warn("Unsupported Base protocol message: 0x%x", msg_id);
        return SCMI_ERR_SUPPORT;
    }
}

/* Base Protocol Operations */
static const struct scmi_protocol base_protocol = {
    .id = SCMI_PROTO_ID_BASE,
    .handle_request = virtio_scmi_base_handle_req,
};

/* Initialize Base Protocol */
int virtio_scmi_base_init(void) {
    return scmi_register_protocol(&base_protocol);
}
