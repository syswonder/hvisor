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
#ifndef _HVISOR_VIRTIO_SCMI_H
#define _HVISOR_VIRTIO_SCMI_H

#include "virtio.h"
#include <stdbool.h>
#include <stdint.h>

/*
 * SCMI Packed Message Header Format
 *
 *  31          28 27          18 17      10 9    8 7             0
 * +--------------+--------------+----------+------+---------------+
 * |   Reserved   |   Token ID   |ProtocolID| Type |  Message ID   |
 * +--------------+--------------+----------+------+---------------+
 */

struct scmi_msg_header {
    uint32_t msg_id : 8;
    uint32_t msg_type : 2;
    uint32_t protocol_id : 8;
    uint32_t token : 10;
    uint32_t reserved : 4;
} __attribute__((packed));

/* -------------------------- SCMI Message Type Values
 * -------------------------- */
#define SCMI_MSG_TYPE_COMMAND 0      /* Command */
#define SCMI_MSG_TYPE_DELAYED_RESP 2 /* Delayed Response */
#define SCMI_MSG_TYPE_NOTIFICATION 3 /* Notification */

static inline void scmi_build_header(struct scmi_msg_header *hdr,
                                     uint8_t protocol_id, uint8_t msg_id,
                                     uint16_t token) {
    *hdr = (struct scmi_msg_header){
        .msg_id = msg_id,
        .msg_type = SCMI_MSG_TYPE_COMMAND,
        .protocol_id = protocol_id,
        .token = token,
    };
}

/* -------------------------- SCMI Protocol IDs -------------------------- */
#define SCMI_PROTO_ID_BASE 0x10  /* Base Protocol */
#define SCMI_PROTO_ID_POWER 0x11 /* Power Domain Protocol */
#define SCMI_PROTO_ID_CLOCK 0x14 /* Clock Protocol */
#define SCMI_PROTO_ID_RESET 0x16 /* Reset Protocol */

/* -------------------------- SCMI Reset Protocol Message IDs ---------- */
#define SCMI_RESET_MSG_RESET_ATTRIBUTES 0x3 /* Reset domain attributes */
#define SCMI_RESET_MSG_RESET 0x4            /* Reset domain */
#define SCMI_RESET_MSG_RESET_NOTIFY 0x5     /* Reset notify */

/* -------------------------- SCMI Power Protocol Message IDs ---------- */
#define SCMI_POWER_MSG_POWER_DOMAIN_ATTRIBUTES 0x3 /* Power domain attr */
#define SCMI_POWER_MSG_POWER_STATE_SET 0x4         /* Set power state */
#define SCMI_POWER_MSG_POWER_STATE_GET 0x5         /* Get power state */
#define SCMI_POWER_MSG_POWER_STATE_NOTIFY 0x6      /* Power state notify */

/* -------------------------- SCMI Common Protocol Message IDs
 * -------------------------- */
#define SCMI_COMMON_MSG_VERSION 0x0             /* Version request */
#define SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES 0x1 /* Protocol attributes */
#define SCMI_COMMON_MSG_MESSAGE_ATTRIBUTES 0x2  /* Protocol msg attributes */
#define SCMI_BASE_MSG_DISCOVER_VENDOR 0x3
#define SCMI_BASE_MSG_DISCOVER_SUB_VENDOR 0x4
#define SCMI_BASE_MSG_DISCOVER_IMPL_VERSION 0x5
#define SCMI_BASE_MSG_DISCOVER_LIST_PROTOCOLS 0x6
#define SCMI_BASE_MSG_DISCOVER_AGENT 0x7
#define SCMI_BASE_MSG_NOTIFY_ERRORS 0x8
#define SCMI_BASE_MSG_SET_DEVICE_PERMISSIONS 0x9
#define SCMI_BASE_MSG_SET_PROTOCOL_PERMISSIONS 0xa
#define SCMI_BASE_MSG_RESET_AGENT_CONFIGURATION 0xb

/* -------------------------- SCMI Clock Protocol Message IDs ---------- */
#define SCMI_CLOCK_MSG_CLK_ATTRIBUTES 0x3 /* Clock protocol attributes */
#define SCMI_CLOCK_MSG_DESCRIBE_RATES 0x4 /* Describe supported rates */
#define SCMI_CLOCK_MSG_RATE_SET 0x5       /* Set clock rate */
#define SCMI_CLOCK_MSG_RATE_GET 0x6       /* Get current clock rate */
#define SCMI_CLOCK_MSG_CONFIG_SET 0x7     /* Enable/disable clock */
#define SCMI_CLOCK_MSG_CONFIG_GET 0x8     /* Get clock enable state */
#define SCMI_CLOCK_MSG_NAME_GET 0x9       /* Get clock name */

/* Base protocol constants */
#define SCMI_BASE_VENDOR_ID_LEN 16
#define SCMI_BASE_SUB_VENDOR_ID_LEN 16
#define SCMI_BASE_IMPL_VERSION_LEN 4
#define SCMI_BASE_MAX_CMD_ERR_COUNT 5

struct scmi_request {
    struct scmi_msg_header header;
    uint8_t payload[]; /* Message-specific payload */
} __attribute__((packed));

struct scmi_response {
    struct scmi_msg_header header;
    int32_t status;    /* SCMI spec: int32 status code */
    uint8_t payload[]; /* Message-specific payload */
} __attribute__((packed));

/* SCMI Base Protocol Attributes Response */
struct scmi_msg_resp_base_attributes {
    uint8_t num_protocols;
    uint8_t num_agents;
    uint16_t reserved;
} __attribute__((packed));

#define SCMI_MAX_DESCRIPTORS 16
#define SCMI_MAX_BUFFER_SIZE (1024 * 1024) // 1MB
#define SCMI_MAX_PROTOCOLS 16

struct scmi_msg_resp_base_protocol_list {
    uint32_t count;
    uint32_t protocol_slots[(SCMI_MAX_PROTOCOLS + 3) / 4];
} __attribute__((packed));

struct scmi_msg_resp_clock_attributes {
    uint16_t num_clocks;
    uint8_t max_async_req;
    uint8_t reserved;
} __attribute__((packed));

struct scmi_resp_ctx {
    struct iovec *iov; /* underlying response iovec */
    size_t written;    /* bytes written so far */
    size_t capacity;   /* total capacity (original iov_len) */
};

struct virtio_scmi_dev;

struct scmi_dev_protocol_entry {
    uint8_t protocol_id;
    int (*handler)(struct virtio_scmi_dev *dev, uint8_t msg_id, uint16_t token,
                   const struct iovec *req_iov, struct scmi_resp_ctx *ctx);
};

struct virtio_scmi_config {
    uint64_t reserved[8];
};

typedef struct virtio_scmi_dev {
    struct virtio_scmi_config config;
    uint32_t *clock_ids; /* NULL = protocol not supported */
    uint32_t clock_count;
    uint32_t *reset_ids;
    uint32_t reset_count;
    uint32_t *power_ids;
    uint32_t power_count;
    struct scmi_dev_protocol_entry protocols[SCMI_MAX_PROTOCOLS];
    int protocol_count;
} SCMIDev;

enum scmi_error_codes {
    SCMI_SUCCESS = 0,        /* Success */
    SCMI_ERR_SUPPORT = -1,   /* Not supported */
    SCMI_ERR_PARAMS = -2,    /* Invalid Parameters */
    SCMI_ERR_ACCESS = -3,    /* Invalid access/permission denied */
    SCMI_ERR_ENTRY = -4,     /* Not found */
    SCMI_ERR_RANGE = -5,     /* Value out of range */
    SCMI_ERR_BUSY = -6,      /* Device busy */
    SCMI_ERR_COMMS = -7,     /* Communication Error */
    SCMI_ERR_GENERIC = -8,   /* Generic Error */
    SCMI_ERR_HARDWARE = -9,  /* Hardware Error */
    SCMI_ERR_PROTOCOL = -10, /* Protocol Error */
};

#define SCMI_SUPPORTED_FEATURES (1ULL << VIRTIO_F_VERSION_1)

/* Per-device protocol registration */
int scmi_dev_register_protocol(SCMIDev *dev, uint8_t protocol_id,
                               int (*handler)(SCMIDev *dev, uint8_t msg_id,
                                              uint16_t token,
                                              const struct iovec *req_iov,
                                              struct scmi_resp_ctx *ctx));

/* Notification support */
#define SCMI_MAX_QUEUES 2
#define VIRTQUEUE_SCMI_MAX_SIZE 64
#define SCMI_QUEUE_TX 0
#define SCMI_QUEUE_RX 1

/* Base Protocol Events */
#define SCMI_BASE_ERROR_EVENT 0x1
#define BASE_TP_NOTIFY_ALL (1 << 0)

struct scmi_base_error_notify_payld {
    uint32_t agent_id;
    uint32_t error_status;
    uint64_t msg_reports[SCMI_BASE_MAX_CMD_ERR_COUNT];
} __attribute__((packed));

struct scmi_base_error_report {
    uint64_t timestamp;
    uint32_t agent_id;
    bool fatal;
    uint32_t cmd_count;
    uint64_t reports[SCMI_BASE_MAX_CMD_ERR_COUNT];
} __attribute__((packed));

void scmi_resp_ctx_init(struct scmi_resp_ctx *ctx, struct iovec *resp_iov);

void *scmi_resp_write(struct scmi_resp_ctx *ctx, size_t size);

/* Protocol handler entry points */
int virtio_scmi_base_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                const struct iovec *req_iov,
                                struct scmi_resp_ctx *ctx);
int virtio_scmi_clock_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx);
int virtio_scmi_power_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx);
int virtio_scmi_reset_handle_req(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                                 const struct iovec *req_iov,
                                 struct scmi_resp_ctx *ctx);

/* Core SCMI Functions */
int scmi_validate_request(size_t req_size, size_t min_req_size,
                          size_t resp_size, size_t min_resp_size);

int scmi_make_response(struct scmi_resp_ctx *ctx, uint8_t protocol_id,
                       uint8_t msg_id, uint16_t token, int32_t status);

int scmi_handle_message(SCMIDev *dev, uint8_t protocol_id, uint8_t msg_id,
                        uint16_t token, const struct iovec *req_iov,
                        struct scmi_resp_ctx *ctx);

struct virtio_scmi_init_params {
    uint32_t *clock_ids;
    uint32_t clock_count;
    uint32_t *reset_ids;
    uint32_t reset_count;
    uint32_t *power_ids;
    uint32_t power_count;
};

SCMIDev *scmi_dev_create(void);
void scmi_dev_free(SCMIDev *dev);
int virtio_scmi_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);
void virtio_scmi_close(VirtIODevice *vdev);
void virtio_scmi_reset(VirtIODevice *vdev);

extern const struct virtio_device_ops virtio_scmi_ops;

/* JSON array parsing: fills dev->clock_ids / dev->reset_ids / dev->power_ids */
int scmi_dev_parse_clock_ids(struct virtio_scmi_init_params *p,
                             void *json_array);
int scmi_dev_parse_reset_ids(struct virtio_scmi_init_params *p,
                             void *json_array);
int scmi_dev_parse_power_ids(struct virtio_scmi_init_params *p,
                             void *json_array);
void scmi_dev_free_params(struct virtio_scmi_init_params *p);

/* /dev/hvisor fd, opened once in virtio_start() */
extern int ko_fd;

/* All SCMI protocol args structs share {subcmd, data_len} at offset 0. */
struct hvisor_scmi_ioctl_hdr {
    uint32_t subcmd;
    uint32_t data_len;
};

/* Unified ioctl helper using the persistent ko_fd */
int hvisor_scmi_ioctl_cmd(int ioctl_cmd, void *args, size_t args_size,
                          uint32_t subcmd, const char *proto_name);

#endif
