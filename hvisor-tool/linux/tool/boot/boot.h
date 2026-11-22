// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      yyda <snowmantin@foxmail.com>
 */
#ifndef __HVISOR_TOOL_BOOT_H
#define __HVISOR_TOOL_BOOT_H

#include <stdint.h>

#include "hvisor.h"

struct cJSON;

enum boot_mode_kind {
    BOOT_MODE_NONE = 0,
    BOOT_MODE_MULTIBOOT2,
};

struct boot_mode {
    enum boot_mode_kind kind;
    __u64 multiboot_info_paddr;
};

int boot_mode_parse(struct boot_mode *mode, struct cJSON *root);
int boot_mode_is_multiboot2(struct cJSON *root);
int boot_mode_prepare_zone(struct boot_mode *mode, zone_config_t *config,
                           struct cJSON *root);
int boot_mode_apply(int fd, uint32_t zone_id, const struct boot_mode *mode);

#endif /* __HVISOR_TOOL_BOOT_H */
