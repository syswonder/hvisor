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
#include "log.h"
#include "virtio.h"
#include <stdlib.h>
#include <string.h>
#include <errno.h>

SCMIDev *init_scmi_dev() {
    SCMIDev *dev = (SCMIDev *)malloc(sizeof(SCMIDev));
    if (dev) {
        dev->fd = -1;
        /* Initialize supported protocols */
        extern int virtio_scmi_base_init(void);
        extern int virtio_scmi_clock_init(void);
        extern int virtio_scmi_reset_init(void);
        virtio_scmi_base_init();
        virtio_scmi_clock_init();
        virtio_scmi_reset_init();
    }
    return dev;
}

static int virtq_tx_handle_one_request(void *dev, VirtQueue *vq) {
    uint16_t desc_idx;
    struct iovec *iov = NULL;
    uint16_t *flags = NULL;
    int ret = process_descriptor_chain(vq, &desc_idx, &iov, &flags, SCMI_MAX_DESCRIPTORS, true);

    if (ret <= 0 || iov == NULL || flags == NULL) {
        log_error("Failed to process descriptor chain or allocate memory");
        goto error;
    }

    // Check buffer sizes and pointers - accept 4-byte packed header
    if (ret < 2 || iov[0].iov_len < sizeof(uint32_t) ||
        iov[0].iov_base == NULL || iov[0].iov_len > SCMI_MAX_BUFFER_SIZE) {
        log_error("Invalid request buffer");
        goto error;
    }

    // Parse packed 32-bit header
    uint32_t packed_header = *(uint32_t *)iov[0].iov_base;
    uint8_t protocol_id = SCMI_PROTOCOL_ID(packed_header);
    uint8_t msg_id = SCMI_MSG_ID(packed_header);
    uint8_t msg_type = SCMI_MSG_TYPE(packed_header);
    uint16_t token = SCMI_TOKEN_ID(packed_header);

    log_debug("SCMI request: protocol=0x%x, msg=0x%x, type=%d, token=0x%x",
            protocol_id, msg_id, msg_type, token);

    // Validate message type
    if (msg_type != SCMI_MSG_TYPE_COMMAND) {
        log_error("Invalid message type: %d", msg_type);
        goto error;
    }

    // Dispatch request to protocol handler
    int status = scmi_handle_message(dev, protocol_id, msg_id, token,
                                   &iov[0], &iov[1]);

    if (status != 0) {
        log_error("Protocol handler failed: %d", status);
        goto error;
    }

    // Update used ring with total length (request + response)
    update_used_ring(vq, desc_idx, iov[0].iov_len + iov[1].iov_len);
    free(iov);
    free(flags);
    return 0;

error:
    free(iov);
    free(flags);
    return -EINVAL;
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