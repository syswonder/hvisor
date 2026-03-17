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
#include <string.h>

static struct scmi_protocol* protocols[SCMI_MAX_PROTOCOLS];
static int protocol_count = 0;

/* Get protocol by ID */
const struct scmi_protocol *scmi_get_protocol_by_id(uint8_t protocol_id) {
    for (int i = 0; i < protocol_count; i++) {
        if (protocols[i]->id == protocol_id) {
            return protocols[i];
        }
    }
    return NULL;
}

/* Get protocol by index */
const struct scmi_protocol *scmi_get_protocol_by_index(int index) {
    if (index < 0 || index >= protocol_count) {
        return NULL;
    }
    return protocols[index];
}

int scmi_get_protocol_count(void) {
    return protocol_count;
}

/* Validate request/response buffers */
int scmi_validate_request(size_t req_size, size_t min_req_size,
                         size_t resp_size, size_t min_resp_size)
{
    if (req_size < min_req_size) {
        log_error("Request too small: %zu < %zu", req_size, min_req_size);
        return SCMI_ERR_PARAMS;
    }

    if (resp_size < min_resp_size) {
        log_error("Response too small: %zu < %zu", resp_size, min_resp_size);
        return SCMI_ERR_PARAMS;
    }

    return SCMI_SUCCESS;
}

/* Create standard SCMI response */
int scmi_make_response(SCMIDev *dev, uint16_t token,
                      struct iovec *resp_iov, int32_t status)
{
    if (resp_iov->iov_len < sizeof(struct scmi_response)) {
        return SCMI_ERR_PARAMS;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    resp->header = SCMI_RESP_HDR(token);
    resp->status = status;

    return 0;
}

/* Register a protocol implementation */
int scmi_register_protocol(const struct scmi_protocol *proto) {
    if (!proto) {
        log_error("Cannot register NULL protocol");
        return SCMI_ERR_PARAMS;
    }

    /* Check if protocol ID already exists */
    for (int i = 0; i < protocol_count; i++) {
        if (protocols[i]->id == proto->id) {
            log_error("Protocol %u already registered", proto->id);
            return SCMI_ERR_ENTRY;
        }
    }

    if (protocol_count >= SCMI_MAX_PROTOCOLS) {
        log_error("Cannot register protocol - table full");
        return SCMI_ERR_ENTRY;
    }

    protocols[protocol_count] = proto;

    log_info("Registered protocol %u at index %d", proto->id, protocol_count);
    protocol_count++;
    return SCMI_SUCCESS;
}

/* Dispatch SCMI message to protocol handler */
int scmi_handle_message(SCMIDev *dev, uint8_t protocol_id, uint8_t msg_id,
                      uint16_t token, const struct iovec *req_iov,
                      struct iovec *resp_iov)
{
    const struct scmi_protocol *proto = scmi_get_protocol_by_id(protocol_id);
    if (!proto) {
        log_warn("Unsupported protocol: %u", protocol_id);
        return SCMI_ERR_SUPPORT;
    }

    return proto->handle_request(dev, msg_id, token, req_iov, resp_iov);
}
