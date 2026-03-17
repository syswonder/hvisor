// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Linkun Chen <lkchen01@foxmail.com>
 */
#ifndef _HVISOR_VIRTIO_SCMI_H
#define _HVISOR_VIRTIO_SCMI_H

#include "virtio.h"
#include <stdint.h>
#include <stdbool.h>

#define NSEC_PER_SEC 1000000000L
#define SCMI_MAX_POLL_TO_NS    (30 * NSEC_PER_SEC)

/*
 * SCMI Packed Message Header Format
 *
 *  31          28 27          18 17      10 9    8 7             0
 * +--------------+--------------+----------+------+---------------+
 * |   Reserved   |   Token ID   |ProtocolID| Type |  Message ID   |
 * +--------------+--------------+----------+------+---------------+
 */

/* -------------------------- Bitfield Operation Macros -------------------------- */

/* Generate mask, e.g. GENMASK(7,0)=0xFF */
#define BITMASK(high, low)   (((1u << ((high) - (low) + 1)) - 1) << (low))

/* Extract field: mask first, then right shift */
#define EXTRACT_BITS(val, high, low)   (((val) & BITMASK((high), (low))) >> (low))

/* Insert field: left shift first, then mask */
#define INSERT_BITS(val, high, low)    (((val) << (low)) & BITMASK((high), (low)))

/* -------------------------- SCMI Header Field Definitions -------------------------- */

/* Message ID: bits [7:0] */
#define SCMI_MSG_ID_LOW    0
#define SCMI_MSG_ID_HIGH   7

/* Message Type: bits [9:8] */
#define SCMI_MSG_TYPE_LOW  8
#define SCMI_MSG_TYPE_HIGH 9

/* Protocol ID: bits [17:10] */
#define SCMI_PROTOCOL_ID_LOW  10
#define SCMI_PROTOCOL_ID_HIGH 17

/* Token ID: bits [27:18] */
#define SCMI_TOKEN_ID_LOW   18
#define SCMI_TOKEN_ID_HIGH  27

/* -------------------------- SCMI Header Parsing Macros -------------------------- */

/* Extract message ID */
#define SCMI_MSG_ID(hdr) \
	EXTRACT_BITS((hdr), SCMI_MSG_ID_HIGH, SCMI_MSG_ID_LOW)

/* Extract message type */
#define SCMI_MSG_TYPE(hdr) \
	EXTRACT_BITS((hdr), SCMI_MSG_TYPE_HIGH, SCMI_MSG_TYPE_LOW)

/* Extract protocol ID */
#define SCMI_PROTOCOL_ID(hdr) \
	EXTRACT_BITS((hdr), SCMI_PROTOCOL_ID_HIGH, SCMI_PROTOCOL_ID_LOW)

/* Extract Token ID */
#define SCMI_TOKEN_ID(hdr) \
	EXTRACT_BITS((hdr), SCMI_TOKEN_ID_HIGH, SCMI_TOKEN_ID_LOW)

/* Construct response header */
#define SCMI_RESP_HDR(token) \
	(INSERT_BITS(0x0, SCMI_MSG_ID_HIGH, SCMI_MSG_ID_LOW) | \
	 INSERT_BITS(SCMI_MSG_TYPE_COMMAND, SCMI_MSG_TYPE_HIGH, SCMI_MSG_TYPE_LOW) | \
	 INSERT_BITS(0x10, SCMI_PROTOCOL_ID_HIGH, SCMI_PROTOCOL_ID_LOW) | \
	 INSERT_BITS((token), SCMI_TOKEN_ID_HIGH, SCMI_TOKEN_ID_LOW))

/* -------------------------- SCMI Message Type Values -------------------------- */
#define SCMI_MSG_TYPE_COMMAND         0  /* Command */
#define SCMI_MSG_TYPE_DELAYED_RESP    2  /* Delayed Response */
#define SCMI_MSG_TYPE_NOTIFICATION    3  /* Notification */

/* -------------------------- SCMI Protocol IDs -------------------------- */
#define SCMI_PROTO_ID_BASE     0x10    /* Base Protocol */
#define SCMI_PROTO_ID_CLOCK    0x14    /* Clock Protocol */
#define SCMI_PROTO_ID_RESET    0x16    /* Reset Protocol */

/* -------------------------- SCMI Reset Protocol Message IDs ---------- */
#define SCMI_RESET_MSG_RESET_ATTRIBUTES         0x3  /* Reset domain attributes */
#define SCMI_RESET_MSG_RESET                     0x4  /* Reset domain */
#define SCMI_RESET_MSG_RESET_NOTIFY              0x5  /* Reset notify */

/* -------------------------- SCMI Common Protocol Message IDs -------------------------- */
#define SCMI_COMMON_MSG_VERSION                        0x0  /* Version request */
#define SCMI_COMMON_MSG_PROTOCOL_ATTRIBUTES            0x1  /* Protocol attributes */
#define SCMI_COMMON_MSG_MESSAGE_ATTRIBUTES             0x2  /* Protocol message attributes */

/* -------------------------- SCMI Base Protocol Message IDs -------------------------- */
#define SCMI_BASE_MSG_DISCOVER_VENDOR           0x3
#define SCMI_BASE_MSG_DISCOVER_SUB_VENDOR       0x4
#define SCMI_BASE_MSG_DISCOVER_IMPL_VERSION     0x5
#define SCMI_BASE_MSG_DISCOVER_LIST_PROTOCOLS   0x6
#define SCMI_BASE_MSG_DISCOVER_AGENT            0x7
#define SCMI_BASE_MSG_NOTIFY_ERRORS             0x8
#define SCMI_BASE_MSG_SET_DEVICE_PERMISSIONS    0x9
#define SCMI_BASE_MSG_SET_PROTOCOL_PERMISSIONS  0xa
#define SCMI_BASE_MSG_RESET_AGENT_CONFIGURATION 0xb

/* -------------------------- SCMI Clock Protocol Message IDs ---------- */
#define SCMI_CLOCK_MSG_CLK_ATTRIBUTES         0x3  /* Clock protocol attributes */
#define SCMI_CLOCK_MSG_DESCRIBE_RATES         0x4  /* Describe supported rates */
#define SCMI_CLOCK_MSG_RATE_SET               0x5  /* Set clock rate */
#define SCMI_CLOCK_MSG_RATE_GET               0x6  /* Get current clock rate */
#define SCMI_CLOCK_MSG_CONFIG_SET             0x7  /* Enable/disable clock */
#define SCMI_CLOCK_MSG_CONFIG_GET             0x8  /* Get clock enable state */
#define SCMI_CLOCK_MSG_NAME_GET               0x9  /* Get clock name */

/* Base protocol constants */
#define SCMI_BASE_VENDOR_ID_LEN         16
#define SCMI_BASE_SUB_VENDOR_ID_LEN     16
#define SCMI_BASE_IMPL_VERSION_LEN      4
#define SCMI_BASE_MAX_CMD_ERR_COUNT     5

struct scmi_request {
    uint32_t header; /* Packed SCMI message header */
    uint8_t payload[]; /* Message-specific payload */
};

struct scmi_response {
    uint32_t header; /* Packed SCMI message header */
    uint32_t status; /* Command status */
    uint8_t payload[]; /* Message-specific payload */
};

/* SCMI Base Protocol Attributes Response */
struct scmi_msg_resp_base_attributes {
    uint8_t num_protocols;
    uint8_t num_agents;
    uint16_t reserved;
};

struct scmi_msg_resp_clock_attributes {
    uint16_t num_clocks;
    uint8_t max_async_req;
    uint8_t reserved;
};

/* -------------------------- Token Maximum Value -------------------------- */
#define SCMI_TOKEN_MAX  ((1u << (SCMI_TOKEN_ID_HIGH - SCMI_TOKEN_ID_LOW + 1)) - 1)

typedef struct virtio_scmi_dev {
    int fd;
} SCMIDev;

/* -------------------------- Protocol Callback Table -------------------------- */
struct virtio_scmi_proto {
    uint8_t proto_id;  /* Protocol ID (e.g. SCMI_PROTO_ID_BASE) */
    /* Handle protocol request: return 0 on success, negative for error */
    int (*handle_req)(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                     const struct iovec *req_iov, struct iovec *resp_iov);
};

enum scmi_error_codes {
	SCMI_SUCCESS = 0,	/* Success */
	SCMI_ERR_SUPPORT = -1,	/* Not supported */
	SCMI_ERR_PARAMS = -2,	/* Invalid Parameters */
	SCMI_ERR_ACCESS = -3,	/* Invalid access/permission denied */
	SCMI_ERR_ENTRY = -4,	/* Not found */
	SCMI_ERR_RANGE = -5,	/* Value out of range */
	SCMI_ERR_BUSY = -6,	/* Device busy */
	SCMI_ERR_COMMS = -7,	/* Communication Error */
	SCMI_ERR_GENERIC = -8,	/* Generic Error */
	SCMI_ERR_HARDWARE = -9,	/* Hardware Error */
	SCMI_ERR_PROTOCOL = -10,/* Protocol Error */
};

#define SCMI_SUPPORTED_FEATURES                                             \
    (1ULL << VIRTIO_F_VERSION_1)

#define SCMI_MAX_DESCRIPTORS     16
#define SCMI_MAX_BUFFER_SIZE    (1024 * 1024) // 1MB
#define SCMI_MAX_PROTOCOLS       16

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
};

struct scmi_base_error_report {
    uint64_t timestamp;
    uint32_t agent_id;
    bool fatal;
    uint32_t cmd_count;
    uint64_t reports[SCMI_BASE_MAX_CMD_ERR_COUNT];
};

/* SCMI Protocol Operations */
struct scmi_protocol {
    uint8_t id;
    
    /* Message handlers */
    int (*handle_request)(SCMIDev *dev, uint8_t msg_id, uint16_t token,
                         const struct iovec *req_iov, struct iovec *resp_iov);
};

/* Core SCMI Functions */
int scmi_validate_request(size_t req_size, size_t min_req_size,
                         size_t resp_size, size_t min_resp_size);

int scmi_make_response(SCMIDev *dev, uint16_t token, 
                      struct iovec *resp_iov, int32_t status);

/* Protocol Registration */
int scmi_register_protocol(const struct scmi_protocol *ops);

const struct scmi_protocol *scmi_get_protocol_by_index(int index);

int scmi_handle_message(SCMIDev *dev, uint8_t protocol_id, uint8_t msg_id,
                      uint16_t token, const struct iovec *req_iov,
                      struct iovec *resp_iov);

int scmi_get_protocol_count(void);

SCMIDev *init_scmi_dev();
int virtio_scmi_txq_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

#endif