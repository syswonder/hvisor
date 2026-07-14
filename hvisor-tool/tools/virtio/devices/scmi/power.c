// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      hvisor-tool contributors
 */

#include "hvisor.h"
#include "log.h"
#include "virtio_scmi.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Power Protocol version 2.0 */
#define SCMI_POWER_VERSION 0x20000

#pragma GCC diagnostic ignored "-Wunused-parameter"

/* Response for POWER_DOMAIN_ATTRIBUTES (Message ID 0x3) */
struct scmi_msg_resp_power_domain_attributes {
    uint32_t flags;
    char name[16];
};

/* Request for POWER_STATE_SET (Message ID 0x4) */
struct scmi_msg_req_power_state_set {
    uint32_t flags;
    uint32_t domain_id;
    uint32_t power_state;
};

/* Helper: validate power domain_id and get physical ID */
static uint32_t pwr_phys_id(SCMIDev *dev, uint32_t dom_id, bool *valid) {
    if (!dev->power_ids) {
        *valid = false;
        return 0;
    }
    if (dom_id >= dev->power_count) {
        *valid = false;
        return 0;
    }
    *valid = true;
    return dev->power_ids[dom_id];
}

/* ================ Handlers ================ */

static int handle_power_version(SCMIDev *dev, uint16_t token,
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
    scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_COMMON_MSG_VERSION,
                       token, SCMI_SUCCESS);
    *version = SCMI_POWER_VERSION;
    return 0;
}

static int handle_power_protocol_attributes(SCMIDev *dev, uint16_t token,
                                            const struct iovec *req_iov,
                                            struct iovec *resp_iov) {
    int ret = scmi_validate_request(
        req_iov->iov_len, sizeof(struct scmi_request), resp_iov->iov_len,
        sizeof(struct scmi_response) + sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *attributes = (uint32_t *)resp->payload;

    *attributes = (uint32_t)(dev->power_count & 0xFFFF);

    scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                       SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES, token,
                       SCMI_SUCCESS);

    log_debug("POWER_PROTOCOL_ATTRIBUTES: num_domains=%d", dev->power_count);
    return 0;
}

static int handle_power_domain_attributes(SCMIDev *dev, uint16_t token,
                                          const struct iovec *req_iov,
                                          struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;
    bool valid;
    uint32_t phys_id = pwr_phys_id(dev, domain_id, &valid);

    if (!valid) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token,
                                  SCMI_ERR_ENTRY);
    }

    size_t expected_resp_size =
        sizeof(struct scmi_response) +
        sizeof(struct scmi_msg_resp_power_domain_attributes);
    if (resp_iov->iov_len < expected_resp_size) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_power_args args;
    args.u.power_attr.domain_id = phys_id;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_POWER_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_POWER_GET_ATTRIBUTES, "power");
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token,
                                  SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_power_domain_attributes *attr =
        (struct scmi_msg_resp_power_domain_attributes *)resp->payload;

    attr->flags = (1U << 29); /* SYNCHRONOUS support only */
    strncpy(attr->name, args.u.power_attr.name, 15);
    attr->name[15] = '\0';

    log_debug("POWER_DOMAIN_ATTRIBUTES: domain=%u, name=%s, flags=0x%x",
              domain_id, attr->name, attr->flags);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                              SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token,
                              SCMI_SUCCESS);
}

static int handle_power_state_set(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_power_state_set *power_req =
        (struct scmi_msg_req_power_state_set *)req->payload;

    uint32_t flags = power_req->flags;
    uint32_t domain_id = power_req->domain_id;
    uint32_t power_state = power_req->power_state;
    bool valid;
    uint32_t phys_id = pwr_phys_id(dev, domain_id, &valid);

    if (!valid) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_SET, token,
                                  SCMI_ERR_ENTRY);
    }

    if (flags & 0x1) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_SET, token,
                                  SCMI_ERR_SUPPORT);
    }

    struct hvisor_scmi_power_args args;
    args.u.power_state_info.domain_id = phys_id;
    args.u.power_state_info.power_state = power_state;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_POWER_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_POWER_STATE_SET, "power");
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_SET, token,
                                  SCMI_ERR_GENERIC);
    }

    log_debug("POWER_STATE_SET: domain=%u, state=0x%x, flags=0x%x", domain_id,
              power_state, flags);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                              SCMI_POWER_MSG_POWER_STATE_SET, token,
                              SCMI_SUCCESS);
}

static int handle_power_state_get(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;
    bool valid;
    uint32_t phys_id = pwr_phys_id(dev, domain_id, &valid);

    if (!valid) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_GET, token,
                                  SCMI_ERR_ENTRY);
    }

    if (resp_iov->iov_len < sizeof(struct scmi_response) + sizeof(uint32_t)) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_GET, token,
                                  SCMI_ERR_RANGE);
    }

    struct hvisor_scmi_power_args args;
    args.u.power_state_info.domain_id = phys_id;

    int ret =
        hvisor_scmi_ioctl_cmd(HVISOR_SCMI_POWER_IOCTL, &args, sizeof(args),
                              HVISOR_SCMI_POWER_STATE_GET, "power");
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                                  SCMI_POWER_MSG_POWER_STATE_GET, token,
                                  SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *state = (uint32_t *)resp->payload;
    *state = args.u.power_state_info.power_state;

    log_debug("POWER_STATE_GET: domain=%u, state=0x%x", domain_id, *state);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                              SCMI_POWER_MSG_POWER_STATE_GET, token,
                              SCMI_SUCCESS);
}

static int handle_power_state_notify(SCMIDev *dev, uint16_t token,
                                     const struct iovec *req_iov
                                     __attribute__((unused)),
                                     struct iovec *resp_iov) {
    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER,
                              SCMI_POWER_MSG_POWER_STATE_NOTIFY, token,
                              SCMI_ERR_SUPPORT);
}

/* Main request handler */
int virtio_scmi_power_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct iovec *resp_iov) {
    if (resp_iov->iov_len < sizeof(struct scmi_response) ||
        resp_iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
    case SCMI_COMMON_MSG_VERSION:
        return handle_power_version(dev, token, req_iov, resp_iov);
    case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
        return handle_power_protocol_attributes(dev, token, req_iov, resp_iov);
    case SCMI_COMMON_MSG_MESSAGE_ATTRIBUTES:
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, msg_id, token,
                                  SCMI_ERR_SUPPORT);
    case SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES:
        return handle_power_domain_attributes(dev, token, req_iov, resp_iov);
    case SCMI_POWER_MSG_POWER_STATE_SET:
        return handle_power_state_set(dev, token, req_iov, resp_iov);
    case SCMI_POWER_MSG_POWER_STATE_GET:
        return handle_power_state_get(dev, token, req_iov, resp_iov);
    case SCMI_POWER_MSG_POWER_STATE_NOTIFY:
        return handle_power_state_notify(dev, token, req_iov, resp_iov);
    default:
        log_warn("Unsupported Power protocol message: 0x%x", msg_id);
        return SCMI_ERR_SUPPORT;
    }
}

static const struct scmi_protocol power_protocol = {
    .id = SCMI_PROTO_ID_POWER,
    .handle_request = virtio_scmi_power_handle_req,
};

int virtio_scmi_power_init(void) {
    return scmi_register_protocol(&power_protocol);
}
