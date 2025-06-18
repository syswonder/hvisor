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

#include "cJSON.h"

cJSON *__safe_cJSON_GetObjectItem(const char *filename, int line,
                                  const cJSON *object, const char *key);
int __safe_cJSON_GetArraySize(const char *filename, int line,
                              const cJSON *array);
cJSON *__safe_cJSON_GetArrayItem(const char *filename, int line,
                                 const cJSON *array, int index);
cJSON *__safe_cJSON_Parse(const char *filename, int line, const char *value);
void __safe_cJSON_Delete(const char *filename, int line, cJSON *object);

#define SAFE_CJSON_GET_OBJECT_ITEM(object, key)                                \
    __safe_cJSON_GetObjectItem(__FILE__, __LINE__, object, key)
#define SAFE_CJSON_GET_ARRAY_SIZE(array)                                       \
    __safe_cJSON_GetArraySize(__FILE__, __LINE__, array)
#define SAFE_CJSON_GET_ARRAY_ITEM(array, index)                                \
    __safe_cJSON_GetArrayItem(__FILE__, __LINE__, array, index)
#define SAFE_CJSON_PARSE(value) __safe_cJSON_Parse(__FILE__, __LINE__, value)
#define SAFE_CJSON_DELETE(object)                                              \
    __safe_cJSON_Delete(__FILE__, __LINE__, object)