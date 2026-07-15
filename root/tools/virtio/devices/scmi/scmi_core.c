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
#include <errno.h>
#include <string.h>
#include <sys/ioctl.h>

/* Validate request/response buffers */
int scmi_validate_request(size_t req_size, size_t min_req_size,
                          size_t resp_size, size_t min_resp_size) {
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

/* Initialize response context from an iovec */
void scmi_resp_ctx_init(struct scmi_resp_ctx *ctx, struct iovec *resp_iov) {
    ctx->iov = resp_iov;
    ctx->written = 0;
    ctx->capacity = resp_iov->iov_len;
}

/* Reserve space in response buffer and return write pointer */
void *scmi_resp_write(struct scmi_resp_ctx *ctx, size_t size) {
    if (!ctx || !ctx->iov) {
        log_error("scmi_resp_write: NULL ctx or iov");
        return NULL;
    }
    if (ctx->written + size > ctx->capacity) {
        log_error(
            "scmi_resp_write: overflow (written=%zu + size=%zu > capacity=%zu)",
            ctx->written, size, ctx->capacity);
        return NULL;
    }
    void *ptr = (uint8_t *)ctx->iov->iov_base + ctx->written;
    ctx->written += size;
    return ptr;
}

/* Create standard SCMI response header + status */
int scmi_make_response(struct scmi_resp_ctx *ctx, uint8_t protocol_id,
                       uint8_t msg_id, uint16_t token, int32_t status) {
    if (ctx->capacity < sizeof(struct scmi_response))
        return SCMI_ERR_PARAMS;

    struct scmi_response *resp = ctx->iov->iov_base;
    resp->header = SCMI_RESP_HDR(protocol_id, msg_id, token);
    resp->status = status;
    ctx->written = sizeof(struct scmi_response);

    return 0;
}

/* Per-device protocol registration */
int scmi_dev_register_protocol(SCMIDev *dev, uint8_t protocol_id,
                               int (*handler)(SCMIDev *dev, uint8_t msg_id,
                                              uint16_t token,
                                              const struct iovec *req_iov,
                                              struct scmi_resp_ctx *ctx)) {
    if (dev->protocol_count >= SCMI_MAX_PROTOCOLS) {
        log_error("scmi_dev_register_protocol: per-device table full (max %d)",
                  SCMI_MAX_PROTOCOLS);
        return -1;
    }
    for (int i = 0; i < dev->protocol_count; i++)
        if (dev->protocols[i].protocol_id == protocol_id) {
            log_warn(
                "scmi_dev_register_protocol: protocol %u already registered",
                protocol_id);
            return 0;
        }
    dev->protocols[dev->protocol_count].protocol_id = protocol_id;
    dev->protocols[dev->protocol_count].handler = handler;
    dev->protocol_count++;
    return 0;
}

/* Dispatch SCMI message to protocol handler — per-device table first, then
 * global */
int scmi_handle_message(SCMIDev *dev, uint8_t protocol_id, uint8_t msg_id,
                        uint16_t token, const struct iovec *req_iov,
                        struct scmi_resp_ctx *ctx) {
    for (int i = 0; i < dev->protocol_count; i++)
        if (dev->protocols[i].protocol_id == protocol_id)
            return dev->protocols[i].handler(dev, msg_id, token, req_iov, ctx);

    log_warn("Unsupported protocol: 0x%x", protocol_id);
    return SCMI_ERR_SUPPORT;
}

/* Unified ioctl helper shared by clock/reset/power userspace.
 * Uses the persistent ko_fd (opened once in virtio_start()).
 * All three hvisor_scmi_*_args structs start with {subcmd, data_len},
 * so a single implementation works for all protocols.
 */
int hvisor_scmi_ioctl_cmd(int ioctl_cmd, void *args, size_t args_size,
                          uint32_t subcmd, const char *proto_name) {
    struct hvisor_scmi_ioctl_hdr *hdr = args;
    hdr->subcmd = subcmd;
    hdr->data_len = args_size - sizeof(struct hvisor_scmi_ioctl_hdr);

    if (ioctl(ko_fd, ioctl_cmd, args) < 0) {
        log_error("Failed to perform SCMI %s ioctl, subcmd=%u: %s", proto_name,
                  subcmd, strerror(errno));
        return -EIO;
    }
    return 0;
}
