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
#include <stdio.h>
#include <stdlib.h>

/* Power Domain map context */
static scmi_map_context_t power_map_ctx = {
    .map = NULL,
    .map_count = 0,
    .allowed_ids = NULL,
    .allowed_count = 0,
    .allow_all = false
};

/* Power Protocol version 2.0 */
#define SCMI_POWER_VERSION 0x20000

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

/**
 * hvisor_scmi_ioctl - Generic hvisor device ioctl wrapper for power operations
 */
static int hvisor_scmi_ioctl(uint32_t subcmd, struct hvisor_scmi_power_args *ioctl_args, size_t args_size) {
    int fd = open(HVISOR_DEVICE, O_RDWR);
    if (fd < 0) {
        log_error("Failed to open hvisor device");
        return -ENODEV;
    }

    ioctl_args->subcmd = subcmd;
    ioctl_args->data_len = args_size - offsetof(struct hvisor_scmi_power_args, u);

    if (ioctl(fd, HVISOR_SCMI_POWER_IOCTL, ioctl_args) < 0) {
        log_error("Failed to perform SCMI power ioctl, subcmd=%u: %s",
                 subcmd, strerror(errno));
        close(fd);
        return -EIO;
    }
    close(fd);

    return 0;
}

/* Helper: validate power domain id */
static bool is_valid_power_id(uint32_t domain_id) {
    if (power_map_ctx.allow_all) {
        extern uint32_t power_max_num;
        return domain_id < power_max_num;
    }
    return scmi_is_valid_id(&power_map_ctx, domain_id);
}

/* ================ Handlers ================ */

static int handle_power_version(SCMIDev *dev, uint16_t token,
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
    scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_COMMON_MSG_VERSION, token, SCMI_SUCCESS);
    *version = SCMI_POWER_VERSION;
    return 0;
}

static int handle_power_protocol_attributes(SCMIDev *dev, uint16_t token,
                                            const struct iovec *req_iov,
                                            struct iovec *resp_iov) {
    int ret = scmi_validate_request(req_iov->iov_len, sizeof(struct scmi_request),
                                  resp_iov->iov_len, sizeof(struct scmi_response) +
                                  sizeof(uint32_t));
    if (ret != SCMI_SUCCESS) {
        log_error("Invalid request/response sizes");
        return ret;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *attributes = (uint32_t *)resp->payload;

    /* Attributes: Bits[15:0] = number of power domains */
    extern uint32_t power_max_num;
    *attributes = (uint32_t)(power_max_num & 0xFFFF);

    scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES, token, SCMI_SUCCESS);

    log_debug("POWER_PROTOCOL_ATTRIBUTES: num_domains=%d", power_max_num);
    return 0;
}

static int handle_power_domain_attributes(SCMIDev *dev, uint16_t token,
                                          const struct iovec *req_iov,
                                          struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;

    if (!is_valid_power_id(domain_id)) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token, SCMI_ERR_ENTRY);
    }

    size_t expected_resp_size = sizeof(struct scmi_response) +
                                sizeof(struct scmi_msg_resp_power_domain_attributes);
    if (resp_iov->iov_len < expected_resp_size) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token, SCMI_ERR_RANGE);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_power_args args;
    args.u.power_attr.domain_id = scmi_map_id(&power_map_ctx, domain_id);

    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_POWER_GET_ATTRIBUTES, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_power_domain_attributes *attr =
        (struct scmi_msg_resp_power_domain_attributes *)resp->payload;

    /* Build flags field:
     * Bit[29]: supports synchronous state set
     * Bit[31]: supports notifications (not supported)
     */
    attr->flags = (1U << 29); /* SYNCHRONOUS support only */

    /* Copy domain name (max 15 chars + null) */
    strncpy(attr->name, args.u.power_attr.name, 15);
    attr->name[15] = '\0';

    log_debug("POWER_DOMAIN_ATTRIBUTES: domain=%u, name=%s, flags=0x%x",
              domain_id, attr->name, attr->flags);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES, token, SCMI_SUCCESS);
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

    if (!is_valid_power_id(domain_id)) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_SET, token, SCMI_ERR_ENTRY);
    }

    /* Reject async mode */
    if (flags & 0x1) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_SET, token, SCMI_ERR_SUPPORT);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_power_args args;
    args.u.power_state_info.domain_id = scmi_map_id(&power_map_ctx, domain_id);
    args.u.power_state_info.power_state = power_state;

    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_POWER_STATE_SET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_SET, token, SCMI_ERR_GENERIC);
    }

    log_debug("POWER_STATE_SET: domain=%u, state=0x%x, flags=0x%x",
              domain_id, power_state, flags);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_SET, token, SCMI_SUCCESS);
}

static int handle_power_state_get(SCMIDev *dev, uint16_t token,
                                  const struct iovec *req_iov,
                                  struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;

    if (!is_valid_power_id(domain_id)) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_GET, token, SCMI_ERR_ENTRY);
    }

    if (resp_iov->iov_len < sizeof(struct scmi_response) + sizeof(uint32_t)) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_GET, token, SCMI_ERR_RANGE);
    }

    /* Prepare ioctl arguments */
    struct hvisor_scmi_power_args args;
    args.u.power_state_info.domain_id = scmi_map_id(&power_map_ctx, domain_id);

    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_POWER_STATE_GET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_GET, token, SCMI_ERR_GENERIC);
    }

    struct scmi_response *resp = resp_iov->iov_base;
    uint32_t *state = (uint32_t *)resp->payload;
    *state = args.u.power_state_info.power_state;

    log_debug("POWER_STATE_GET: domain=%u, state=0x%x",
              domain_id, *state);

    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_GET, token, SCMI_SUCCESS);
}

static int handle_power_state_notify(SCMIDev *dev, uint16_t token,
                                     const struct iovec *req_iov __attribute__((unused)),
                                     struct iovec *resp_iov) {
    /* Power state change notifications are not supported */
    return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, SCMI_POWER_MSG_POWER_STATE_NOTIFY, token, SCMI_ERR_SUPPORT);
}

/* Main request handler */
static int virtio_scmi_power_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov) {
    if (resp_iov->iov_len < sizeof(struct scmi_response) || resp_iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
        case SCMI_COMMON_MSG_VERSION:
            return handle_power_version(dev, token, req_iov, resp_iov);
        case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
            return handle_power_protocol_attributes(dev, token, req_iov, resp_iov);
        case SCMI_COMMON_MSG_MESSAGE_ATTRIBUTES:
            /* Message attributes not implemented - return not supported */
            return scmi_make_response(resp_iov, SCMI_PROTO_ID_POWER, msg_id, token, SCMI_ERR_SUPPORT);
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

/* Power Protocol Operations */
static const struct scmi_protocol power_protocol = {
    .id = SCMI_PROTO_ID_POWER,
    .handle_request = virtio_scmi_power_handle_req,
};

/**
 * virtio_scmi_power_init_map - Initialize power domain allowed list and map
 */
int virtio_scmi_power_init_map(cJSON *allowed_list_json, cJSON *power_map_json) {
    int ret = scmi_init_map(&power_map_ctx, allowed_list_json, power_map_json,
                            "power_ids", "power_map");

    /* Set power provider phandle if available */
    extern uint32_t power_phandle;
    if (power_phandle > 0) {
        int fd = open(HVISOR_DEVICE, O_RDWR);
        if (fd >= 0) {
            struct hvisor_scmi_power_args args;
            args.subcmd = HVISOR_SCMI_POWER_SET_PHANDLE;
            args.data_len = sizeof(args.u.power_phandle_info);
            args.u.power_phandle_info.phandle = power_phandle;
            if (ioctl(fd, HVISOR_SCMI_POWER_IOCTL, &args) < 0) {
                log_error("Failed to set power provider phandle: %s", strerror(errno));
            } else {
                log_info("Power provider phandle set to %u", power_phandle);
            }
            close(fd);
        }
    }

    return ret;
}

/* Initialize Power Protocol */
int virtio_scmi_power_init(void) {
    return scmi_register_protocol(&power_protocol);
}
