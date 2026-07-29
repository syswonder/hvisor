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
#define _GNU_SOURCE

#include "virtio_scmi.h"
#include "json_parse.h"
#include "log.h"
#include "safe_cjson.h"
#include "virtio.h"
#include <errno.h>
#include <stdlib.h>
#include <string.h>

static int parse_id_array(cJSON *json_array, uint32_t **ids_out,
                          uint32_t *count_out) {
    if (!json_array || !cJSON_IsArray(json_array)) {
        *ids_out = NULL;
        *count_out = 0;
        return 0;
    }

    int count = cJSON_GetArraySize(json_array);
    if (count == 0) {
        *ids_out = NULL;
        *count_out = 0;
        return 0;
    }

    uint32_t *ids = malloc(sizeof(uint32_t) * count);
    if (!ids) {
        log_error("Failed to allocate ID array");
        return -ENOMEM;
    }

    for (int i = 0; i < count; i++) {
        cJSON *item = cJSON_GetArrayItem(json_array, i);
        if (parse_json_u32(item, &ids[i]) != 0) {
            log_error("Failed to parse ID at index %d", i);
            free(ids);
            return -EINVAL;
        }
    }

    *ids_out = ids;
    *count_out = (uint32_t)count;
    return 0;
}

SCMIDev *scmi_dev_create(void) { return calloc(1, sizeof(SCMIDev)); }

void scmi_dev_free(SCMIDev *dev) {
    if (!dev)
        return;
    free(dev->clock_ids);
    free(dev->reset_ids);
    free(dev->power_ids);
    free(dev);
}

int scmi_dev_parse_clock_ids(SCMIDev *dev, void *json_array) {
    return parse_id_array((cJSON *)json_array, &dev->clock_ids,
                          &dev->clock_count);
}

int scmi_dev_parse_reset_ids(SCMIDev *dev, void *json_array) {
    return parse_id_array((cJSON *)json_array, &dev->reset_ids,
                          &dev->reset_count);
}

int scmi_dev_parse_power_ids(SCMIDev *dev, void *json_array) {
    return parse_id_array((cJSON *)json_array, &dev->power_ids,
                          &dev->power_count);
}

static int virtq_tx_handle_one_request(void *dev, VirtQueue *vq) {
    struct iovec out_iov[2], in_iov[2];
    struct VirtioBufConfig cfg = {
        .out_iov = out_iov,
        .max_out = 2,
        .in_iov = in_iov,
        .max_in = 2,
    };
    struct VirtioRequest vreq;

    uint16_t desc_idx =
        vq->avail_ring->ring[vq->last_avail_idx & (vq->num - 1)];
    int ret = process_descriptor_chain_buf(vq, desc_idx, &cfg, &vreq);
    if (ret <= 0) {
        log_error("Failed to process descriptor chain");
        return -EINVAL;
    }

    // SCMI expects: one readable (request header+payload) and one writable
    // (response buffer).  More than that is a malformed chain.
    if (vreq.out_count != 1 || vreq.in_count != 1) {
        log_error("Invalid descriptor chain layout: out=%d, in=%d",
                  vreq.out_count, vreq.in_count);
        update_used_ring(vq, desc_idx, 0);
        return -EINVAL;
    }

    struct iovec *req_iov = &vreq.out_iov[0];
    struct iovec *resp_iov = &vreq.in_iov[0];

    // Check the request buffer: must have a 4-byte packed header
    if (req_iov->iov_len < sizeof(uint32_t) || req_iov->iov_base == NULL ||
        req_iov->iov_len > SCMI_MAX_BUFFER_SIZE) {
        log_error("Invalid request buffer");
        update_used_ring(vq, desc_idx, 0);
        return -EINVAL;
    }

    // Parse packed 32-bit header via bitfield struct
    struct scmi_msg_header *hdr = req_iov->iov_base;

    log_debug("SCMI request: protocol=0x%x, msg=0x%x, type=%d, token=0x%x",
              hdr->protocol_id, hdr->msg_id, hdr->msg_type, hdr->token);

    if (hdr->msg_type != SCMI_MSG_TYPE_COMMAND) {
        log_error("Invalid message type: %d", hdr->msg_type);
        update_used_ring(vq, desc_idx, 0);
        return -EINVAL;
    }

    struct scmi_resp_ctx ctx;
    scmi_resp_ctx_init(&ctx, resp_iov);

    if (scmi_handle_message(dev, hdr->protocol_id, hdr->msg_id, hdr->token,
                            req_iov, &ctx) != 0) {
        log_error("Protocol handler failed");
        update_used_ring(vq, desc_idx, 0);
        return -EINVAL;
    }

    update_used_ring(vq, desc_idx, ctx.written);
    return 0;
}

int virtio_scmi_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
    while (!virtqueue_is_empty(vq)) {
        virtqueue_disable_notify(vq);
        while (!virtqueue_is_empty(vq)) {
            if (virtq_tx_handle_one_request(vdev->dev, vq) < 0) {
                log_error("Failed to handle SCMI request");
                return -1;
            }
        }
        virtqueue_enable_notify(vq);
    }
    virtio_inject_irq(vq);
    return 0;
}

void virtio_scmi_close(VirtIODevice *vdev) {
    SCMIDev *dev = vdev->dev;
    scmi_dev_free(dev);
    free(vdev->vqs);
    free(vdev);
}
