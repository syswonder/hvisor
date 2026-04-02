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
#include "safe_cjson.h"
#include <string.h>
#include <stdlib.h>
#include <errno.h>

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

/* Initialize map from JSON configuration */
int scmi_init_map(scmi_map_context_t *ctx, void *allowed_list_json, void *map_json, const char *id_key, const char *map_key) {
    cJSON *allowed_list = (cJSON *)allowed_list_json;
    cJSON *map = (cJSON *)map_json;
    // Initialize allowed IDs
    ctx->allow_all = false;
    if (allowed_list) {
        cJSON *ids_json = SAFE_CJSON_GET_OBJECT_ITEM(allowed_list, id_key);
        if (ids_json) {
            ctx->allowed_count = SAFE_CJSON_GET_ARRAY_SIZE(ids_json);
            if (ctx->allowed_count > 0) {
                ctx->allowed_ids = malloc(sizeof(uint32_t) * ctx->allowed_count);
                if (!ctx->allowed_ids) {
                    log_error("Failed to allocate allowed_ids");
                    return -ENOMEM;
                }
                for (uint32_t i = 0; i < ctx->allowed_count; i++) {
                    cJSON *id_json = SAFE_CJSON_GET_ARRAY_ITEM(ids_json, i);
                    if (id_json->type == cJSON_String && strcmp(id_json->valuestring, "*") == 0) {
                        ctx->allow_all = true;
                        free(ctx->allowed_ids);
                        ctx->allowed_ids = NULL;
                        ctx->allowed_count = 0;
                        log_info("All %s are allowed", id_key);
                        break;
                    }
                    if (id_json->type == cJSON_Number) {
                        ctx->allowed_ids[i] = id_json->valueint;
                    } else {
                        ctx->allowed_ids[i] = strtoul(id_json->valuestring, NULL, 10);
                    }
                }
            }
        }
    }

    // Initialize map
    ctx->map = NULL;
    ctx->map_count = 0;
    if (map) {
        ctx->map_count = cJSON_GetArraySize(map);
        if (ctx->map_count > 0) {
            ctx->map = malloc(sizeof(scmi_map_entry_t) * ctx->map_count);
            if (!ctx->map) {
                log_error("Failed to allocate map");
                return -ENOMEM;
            }
            uint32_t i = 0;
            cJSON *entry = map->child;
            while (entry && i < ctx->map_count) {
                ctx->map[i].scmi_id = strtoul(entry->string, NULL, 10);
                if (entry->type == cJSON_Number) {
                    ctx->map[i].phys_id = entry->valueint;
                } else {
                    ctx->map[i].phys_id = strtoul(entry->valuestring, NULL, 10);
                }
                i++;
                entry = entry->next;
            }
        }
    }

    // If no map provided, use identical mapping
    if (!ctx->map) {
        // If allowed IDs are specified, create identical map for them
        if (ctx->allowed_ids) {
            ctx->map_count = ctx->allowed_count;
            ctx->map = malloc(sizeof(scmi_map_entry_t) * ctx->map_count);
            if (!ctx->map) {
                log_error("Failed to allocate default map");
                return -ENOMEM;
            }
            for (uint32_t i = 0; i < ctx->map_count; i++) {
                ctx->map[i].scmi_id = ctx->allowed_ids[i];
                ctx->map[i].phys_id = ctx->allowed_ids[i];
            }
        }
    }

    log_info("%s initialized with %u entries, allowed %s: %u", 
             map_key, ctx->map_count, id_key, ctx->allowed_count);
    return 0;
}

/* Check if ID is valid */
bool scmi_is_valid_id(scmi_map_context_t *ctx, uint32_t id) {
    if (ctx->allow_all) {
        return true;
    }
    if (!ctx->allowed_ids) {
        return false;
    }
    for (uint32_t i = 0; i < ctx->allowed_count; i++) {
        if (ctx->allowed_ids[i] == id) {
            return true;
        }
    }
    return false;
}

/* Map SCMI ID to physical ID */
uint32_t scmi_map_id(scmi_map_context_t *ctx, uint32_t scmi_id) {
    // If no map, use identical mapping
    if (!ctx->map) {
        return scmi_id;
    }
    // Find in map
    for (uint32_t i = 0; i < ctx->map_count; i++) {
        if (ctx->map[i].scmi_id == scmi_id) {
            return ctx->map[i].phys_id;
        }
    }
    // If not found, use identical mapping
    return scmi_id;
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