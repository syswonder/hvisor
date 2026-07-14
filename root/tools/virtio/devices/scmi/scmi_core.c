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

/* Per-protocol dispatch table — populated by scmi_register_protocol(). */
static const struct scmi_protocol *protocols[SCMI_MAX_PROTOCOLS];
static int protocol_count;

const struct scmi_protocol *scmi_get_protocol_by_id(uint8_t protocol_id) {
    for (int i = 0; i < protocol_count; i++)
        if (protocols[i]->id == protocol_id)
            return protocols[i];
    return NULL;
}

const struct scmi_protocol *scmi_get_protocol_by_index(int index) {
    if (index < 0 || index >= protocol_count)
        return NULL;
    return protocols[index];
}

int scmi_get_protocol_count(void) { return protocol_count; }

int scmi_register_protocol(const struct scmi_protocol *proto) {
    if (!proto)
        return SCMI_ERR_PARAMS;
    for (int i = 0; i < protocol_count; i++)
        if (protocols[i]->id == proto->id)
            return SCMI_ERR_ENTRY;
    if (protocol_count >= SCMI_MAX_PROTOCOLS)
        return SCMI_ERR_ENTRY;
    protocols[protocol_count++] = proto;
    return SCMI_SUCCESS;
}

/* Protocol handler entry points (defined in base.c, clock.c, power.c, reset.c)
 */
extern int virtio_scmi_base_handle_req(SCMIDev *dev, uint8_t msg_id,
                                       uint16_t token,
                                       const struct iovec *req_iov,
                                       struct iovec *resp_iov);
extern int virtio_scmi_clock_handle_req(SCMIDev *dev, uint8_t msg_id,
                                        uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov);
extern int virtio_scmi_power_handle_req(SCMIDev *dev, uint8_t msg_id,
                                        uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov);
extern int virtio_scmi_reset_handle_req(SCMIDev *dev, uint8_t msg_id,
                                        uint16_t token,
                                        const struct iovec *req_iov,
                                        struct iovec *resp_iov);

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

/* Create standard SCMI response */
int scmi_make_response(struct iovec *resp_iov, uint8_t protocol_id,
                       uint8_t msg_id, uint16_t token, int32_t status) {
    if (resp_iov->iov_len < sizeof(struct scmi_response)) {
        return SCMI_ERR_PARAMS;
    }

    struct scmi_response *resp = resp_iov->iov_base;
    resp->header = SCMI_RESP_HDR(protocol_id, msg_id, token);
    resp->status = status;

    return 0;
}

/* Dispatch SCMI message to protocol handler via registration table */
int scmi_handle_message(SCMIDev *dev, uint8_t protocol_id, uint8_t msg_id,
                        uint16_t token, const struct iovec *req_iov,
                        struct iovec *resp_iov) {
    const struct scmi_protocol *proto = scmi_get_protocol_by_id(protocol_id);
    if (!proto) {
        log_warn("Unsupported protocol: 0x%x", protocol_id);
        return SCMI_ERR_SUPPORT;
    }
    return proto->handle_request(dev, msg_id, token, req_iov, resp_iov);
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
