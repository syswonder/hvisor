#include "json_parse.h"

#include <errno.h>
#include <inttypes.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Parse a JSON string to an uintmax_t value with auto base detection.
 *
 * Using uintmax_t ensures maximum portability across all platforms.
 *
 * @param json_str The JSON string to parse.
 * @param result Pointer to store the parsed value.
 * @return 0 on success, -1 on failure (invalid format or overflow).
 */
static int parse_json_number_string(const cJSON *const json_str,
                                    uintmax_t *const result) {
    if (!json_str || !cJSON_IsString(json_str) || !json_str->valuestring ||
        !result) {
        return -1;
    }

    const char *str = json_str->valuestring;
    char *endptr = 0;

    // Clear errno before calling strtoumax
    errno = 0;
    uintmax_t val = strtoumax(str, &endptr, 0);

    // Check for various possible errors
    if (errno == ERANGE) {
        return -1; // Overflow occurred
    }
    if (endptr == str) {
        return -1; // No digits were found
    }
    if (*endptr != '\0') {
        return -1; // String contains trailing invalid characters
    }

    // Success, store the parsed value in the result pointer
    *result = val;
    return 0;
}

/**
 * @brief Parse a JSON item to a uintmax_t value.
 *
 * This function converts a JSON item (which can be either a number or a
 * string) to a uintmax_t value. It prevents negative numbers from wrapping
 * around.
 */
static int parse_json_number(const cJSON *const json_item,
                             uintmax_t *const result) {
    if (!json_item || !result) {
        return -1;
    }

    if (cJSON_IsNumber(json_item)) {
        if (json_item->valueint < 0) {
            // Prevent negative numbers from wrapping around
            return -1;
        }
        *result = (uintmax_t)json_item->valueint;
        return 0;
    }

    if (cJSON_IsString(json_item)) {
        return parse_json_number_string(json_item, result);
    }

    return -1;
}

#define DEFINE_PARSE_JSON_FUNC(name, type, max_val)                            \
    int name(const cJSON *const json_item, type *const result) {               \
        uintmax_t val;                                                         \
        if (!result || parse_json_number(json_item, &val) != 0) {              \
            return -1;                                                         \
        }                                                                      \
        if (val > (max_val)) {                                                 \
            return -1;                                                         \
        }                                                                      \
        *result = (type)val;                                                   \
        return 0;                                                              \
    }

DEFINE_PARSE_JSON_FUNC(parse_json_size, size_t, SIZE_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_address, uintptr_t, UINTPTR_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_u64, uint64_t, UINT64_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_u32, uint32_t, UINT32_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_u8, uint8_t, UINT8_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_linux_u64, __u64, UINT64_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_linux_u32, __u32, UINT32_MAX)
DEFINE_PARSE_JSON_FUNC(parse_json_linux_u8, __u8, UINT8_MAX)
