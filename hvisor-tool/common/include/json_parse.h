#pragma once

#ifndef JSON_PARSE_H
#define JSON_PARSE_H

#include "cJSON.h"

#include <linux/types.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Parse a JSON item to a size_t value.
 *
 * This function converts a JSON item (number or string) to a size_t value.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed size_t value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_size(const cJSON *const json_item, size_t *const result);

/**
 * @brief Parse a JSON item to a uintptr_t value.
 *
 * This function converts a JSON item (number or string) to a uintptr_t value.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed uintptr_t value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_address(const cJSON *const json_item, uintptr_t *const result);

/**
 * @brief Parse a JSON item to a uint64_t value.
 *
 * This function converts a JSON item (number or string) to a uint64_t value.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed uint64_t value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_u64(const cJSON *const json_item, uint64_t *const result);

/**
 * @brief Parse a JSON item to a uint32_t value.
 *
 * This function converts a JSON item (number or string) to a uint32_t value.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed uint32_t value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_u32(const cJSON *const json_item, uint32_t *const result);

/**
 * @brief Parse a JSON item to a uint8_t value.
 *
 * This function converts a JSON item (number or string) to a uint8_t value.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed uint8_t value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_u8(const cJSON *const json_item, uint8_t *const result);

/**
 * @brief Parse a JSON item to a __u64 value.
 *
 * This function converts a JSON item (number or string) to a __u64 value.
 * It ensures strict pointer type compatibility with Linux types.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed __u64 value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_linux_u64(const cJSON *const json_item, __u64 *const result);

/**
 * @brief Parse a JSON item to a __u32 value.
 *
 * This function converts a JSON item (number or string) to a __u32 value.
 * It ensures strict pointer type compatibility with Linux types.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed __u32 value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_linux_u32(const cJSON *const json_item, __u32 *const result);

/**
 * @brief Parse a JSON item to a __u8 value.
 *
 * This function converts a JSON item (number or string) to a __u8 value.
 * It ensures strict pointer type compatibility with Linux types.
 *
 * @param json_item The JSON item to parse.
 * @param result Pointer to store the parsed __u8 value.
 * @return 0 on success, -1 on failure.
 */
int parse_json_linux_u8(const cJSON *const json_item, __u8 *const result);

#ifdef __cplusplus
}
#endif

#endif /* JSON_PARSE_H */
