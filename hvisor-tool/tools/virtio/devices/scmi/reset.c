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
#include <stdio.h>
#include <stdlib.h>

// Reset map context
static scmi_map_context_t reset_map_ctx = {
    .map = NULL,
    .map_count = 0,
    .allowed_ids = NULL,
    .allowed_count = 0,
    .allow_all = false
};

/* Reset Protocol version 2.0 */
#define SCMI_RESET_VERSION 0x20000

/* Response for RESET_ATTRIBUTES (Message ID 0x3) */
struct scmi_msg_resp_reset_attributes {
    uint32_t attributes;          /* Bitfield as per spec */
    uint32_t reset_latency;       /* in microseconds */
    char reset_name[16];          /* Null-terminated, max 15 chars + \0 */
};

/* Request for RESET (Message ID 0x4) */
struct scmi_msg_req_reset {
    uint32_t domain_id;
    uint32_t flags;               /* Bit 0: async, Bit 1: assert, Bit 2: deassert */
    uint32_t reset_state;          /* 0: arch reset, 1: impl defined */
};

/**
 * hvisor_scmi_ioctl - Generic hvisor device ioctl operation wrapper function
 * @subcmd: HVISOR_SCMI_RESET_XXX subcommand
 * @ioctl_args: Input/output parameter structure pointer
 * @args_size: Parameter structure size
 *
 * Return: 0 on success, negative error code on failure
 */
static int hvisor_scmi_ioctl(uint32_t subcmd, struct hvisor_scmi_reset_args *ioctl_args, size_t args_size) {
    int fd = open(HVISOR_DEVICE, O_RDWR);
    if (fd < 0) {
        log_error("Failed to open hvisor device");
        return -ENODEV;
    }

    // Initialize parameters
    ioctl_args->subcmd = subcmd;
    ioctl_args->data_len = args_size - offsetof(struct hvisor_scmi_reset_args, u);

    // Execute ioctl call
    if (ioctl(fd, HVISOR_SCMI_RESET_IOCTL, ioctl_args) < 0) {
        log_error("Failed to perform SCMI reset ioctl, subcmd=%u: %s",
                 subcmd, strerror(errno));
        close(fd);
        return -EIO;
    }
    close(fd);

    return 0;
}

/**
 * virtio_scmi_reset_init_map - Initialize reset map from configuration
 * @allowed_list_json: JSON object containing allowed reset IDs
 * @reset_map_json: JSON object containing reset ID mappings
 *
 * Return: 0 on success, negative error code on failure
 */
int virtio_scmi_reset_init_map(cJSON *allowed_list_json, cJSON *reset_map_json) {
    return scmi_init_map(&reset_map_ctx, allowed_list_json, reset_map_json, "reset_ids", "reset_map");
}

/* Helper: validate reset domain id */
static bool is_valid_reset_id(uint32_t reset_id) {
    return scmi_is_valid_id(&reset_map_ctx, reset_id);
}

static int handle_reset_version(SCMIDev *dev, uint16_t token,
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
    *version = SCMI_RESET_VERSION;
    return 0;
}

static int handle_reset_protocol_attributes(SCMIDev *dev, uint16_t token,
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

    // Set attributes: Bits[15:0] = number of reset domains, Bits[31:16] = 0
    uint32_t num_resets = reset_map_ctx.allow_all ? 0xFFFF : (reset_map_ctx.allowed_ids ? reset_map_ctx.allowed_count : reset_map_ctx.map_count);
    *attributes = (uint32_t)(num_resets & 0xFFFF);

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);

    log_debug("RESET_PROTOCOL_ATTRIBUTES: num_resets=%d", num_resets);
    return 0;
}

static int handle_reset_attributes(SCMIDev *dev, uint16_t token,
                                const struct iovec *req_iov,
                                struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    uint32_t domain_id = *(uint32_t *)req->payload;
    uint32_t phys_rst_id;

    if (!is_valid_reset_id(domain_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    // Validate response buffer size
    size_t expected_resp_size = sizeof(struct scmi_response) +
                                sizeof(struct scmi_msg_resp_reset_attributes);
    if (resp_iov->iov_len < expected_resp_size) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_RANGE);
    }

    // Map SCMI reset ID to physical reset ID
    phys_rst_id = scmi_map_id(&reset_map_ctx, domain_id);

    struct scmi_response *resp = resp_iov->iov_base;
    struct scmi_msg_resp_reset_attributes *attr =
        (struct scmi_msg_resp_reset_attributes *)resp->payload;

    // Set attributes
    attr->attributes = 0;      // Asynchronous reset and reset notifications are not supported
    attr->reset_latency = 100; // 100ms

    // Copy reset domain name (max 15 chars + null)
    snprintf(attr->reset_name, 15, "rst_%u", phys_rst_id);
    attr->reset_name[15] = '\0';

    log_debug("RESET_RESET_ATTRIBUTES: domain_id=%u, phys_id=%u, name=%s, attributes=0x%x, latency=%u",
             domain_id, phys_rst_id, attr->reset_name, attr->attributes, attr->reset_latency);

    scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
    return 0;
}

static int handle_reset(SCMIDev *dev, uint16_t token,
                        const struct iovec *req_iov,
                        struct iovec *resp_iov) {
    struct scmi_request *req = req_iov->iov_base;
    struct scmi_msg_req_reset *reset_req = (struct scmi_msg_req_reset *)req->payload;

    uint32_t domain_id = reset_req->domain_id;
    uint32_t flags = reset_req->flags;
    uint32_t reset_state = reset_req->reset_state;

    if (!is_valid_reset_id(domain_id)) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_ENTRY);
    }

    bool async = (flags & (1 << 2)) != 0;
    // For simplicity, we only support synchronous mode
    if (async) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_SUPPORT);
    }

    // Prepare ioctl arguments
    struct hvisor_scmi_reset_args args;
    // Map SCMI reset ID to physical reset ID
    args.u.reset_info.domain_id = scmi_map_id(&reset_map_ctx, domain_id);
    args.u.reset_info.flags = flags;
    args.u.reset_info.reset_state = reset_state;

    // Call the common ioctl function
    int ret = hvisor_scmi_ioctl(HVISOR_SCMI_RESET_RESET, &args, sizeof(args));
    if (ret < 0) {
        return scmi_make_response(dev, token, resp_iov, SCMI_ERR_GENERIC);
    }

    log_warn("RESET: domain_id=%u, flags=0x%x, reset_state=%u",
             domain_id, flags, reset_state);

    return scmi_make_response(dev, token, resp_iov, SCMI_SUCCESS);
}

static int handle_reset_notify(SCMIDev *dev, uint16_t token,
                              const struct iovec *req_iov __attribute__((unused)),
                              struct iovec *resp_iov) {
    // For simplicity, we don't support reset notifications
    return scmi_make_response(dev, token, resp_iov, SCMI_ERR_SUPPORT);
}

/* Main request handler */
static int virtio_scmi_reset_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                     const struct iovec *req_iov, struct iovec *resp_iov) {
    if (resp_iov->iov_len < sizeof(struct scmi_response) || resp_iov->iov_base == NULL) {
        log_error("Invalid response buffer");
        return SCMI_ERR_PARAMS;
    }

    switch (msg_id) {
        case SCMI_COMMON_MSG_VERSION:
            return handle_reset_version(dev, token, req_iov, resp_iov);
        case SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES:
            return handle_reset_protocol_attributes(dev, token, req_iov, resp_iov);
        case SCMI_RESET_MSG_RESET_ATTRIBUTES:
            return handle_reset_attributes(dev, token, req_iov, resp_iov);
        case SCMI_RESET_MSG_RESET:
            return handle_reset(dev, token, req_iov, resp_iov);
        case SCMI_RESET_MSG_RESET_NOTIFY:
            return handle_reset_notify(dev, token, req_iov, resp_iov);
        default:
            log_warn("Unsupported Reset protocol message: 0x%x", msg_id);
            return SCMI_ERR_SUPPORT;
    }
}

/* Reset Protocol Operations */
static const struct scmi_protocol reset_protocol = {
    .id = SCMI_PROTO_ID_RESET,
    .handle_request = virtio_scmi_reset_handle_req,
};

/* Initialize Reset Protocol */
int virtio_scmi_reset_init(void) {
    return scmi_register_protocol(&reset_protocol);
}