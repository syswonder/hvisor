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

#include "safe_cjson.h"
#include "log.h"

cJSON *__safe_cJSON_GetObjectItem(const char *filename, int line,
                                  const cJSON *object, const char *key) {
    if (!object) {
        log_warn("%s:%d - [cJSON_GetObjectItem] JSON object is NULL", filename,
                 line);
        return NULL;
    }
    cJSON *item = cJSON_GetObjectItem(object, key);
    if (!item) {
        log_warn(
            "%s:%d - [cJSON_GetObjectItem] Key '%s' not found in JSON object",
            filename, line, key);
    }
    return item;
}

int __safe_cJSON_GetArraySize(const char *filename, int line,
                              const cJSON *array) {
    if (!array) {
        log_warn("%s:%d - [cJSON_GetArraySize] Array is NULL", filename, line);
        return 0;
    }
    return cJSON_GetArraySize(array);
}

cJSON *__safe_cJSON_GetArrayItem(const char *filename, int line,
                                 const cJSON *array, int index) {
    if (!array) {
        log_warn("%s:%d - [cJSON_GetArrayItem] Array is NULL", filename, line);
        return NULL;
    }

    const int size = cJSON_GetArraySize(array);
    if (index < 0 || index >= size) {
        log_warn(
            "%s:%d - [cJSON_GetArrayItem] Index %d out of bounds (size: %d)",
            filename, line, index, size);
        return NULL;
    }
    return cJSON_GetArrayItem(array, index);
}

cJSON *__safe_cJSON_Parse(const char *filename, int line, const char *value) {
    cJSON *obj = cJSON_Parse(value);
    if (!obj) {
        log_warn("%s:%d - [cJSON_Parse] Failed to parse JSON file", filename,
                 line);
    }
    return obj;
}

void __safe_cJSON_Delete(const char *filename, int line, cJSON *object) {
    if (!object) {
        log_warn("%s:%d - [cJSON_Delete] Delete NULL object", filename, line);
        return;
    }
    cJSON_Delete(object);
}