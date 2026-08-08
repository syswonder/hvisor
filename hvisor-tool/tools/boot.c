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
#include "boot.h"
#include "log.h"
#include "multiboot2.h"

int boot_mode_parse(struct boot_mode *mode, struct cJSON *root) {
    *mode = (struct boot_mode){0};
    if (multiboot2_enabled(root)) {
        mode->kind = BOOT_MODE_MULTIBOOT2;
    }
    return 0;
}

int boot_mode_is_multiboot2(struct cJSON *root) {
    return multiboot2_enabled(root);
}

int boot_mode_prepare_zone(struct boot_mode *mode, zone_config_t *config,
                           struct cJSON *root) {
    switch (mode->kind) {
    case BOOT_MODE_NONE:
        return 0;
    case BOOT_MODE_MULTIBOOT2:
        return multiboot2_prepare_zone(config, root,
                                       &mode->multiboot_info_paddr);
    }

    log_error("Unsupported boot mode: %d\n", mode->kind);
    return -1;
}

int boot_mode_apply(int fd, uint32_t zone_id, const struct boot_mode *mode) {
    switch (mode->kind) {
    case BOOT_MODE_NONE:
        return 0;
    case BOOT_MODE_MULTIBOOT2:
        return multiboot2_set_boot_mode(fd, zone_id,
                                        mode->multiboot_info_paddr);
    }

    log_error("Unsupported boot mode: %d\n", mode->kind);
    return -1;
}
