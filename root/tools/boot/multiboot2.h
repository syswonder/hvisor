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
#ifndef __MULTIBOOT2_H
#define __MULTIBOOT2_H

#include <stdint.h>

#include "hvisor.h"

struct cJSON;

int multiboot2_enabled(struct cJSON *root);
int load_elf_kernel(const char *elf_path, uint64_t *entry_point,
                    uint64_t *total_size, int64_t gpa_to_hpa_offset);
int multiboot2_prepare_zone(zone_config_t *config, struct cJSON *root,
                            __u64 *multiboot_info_paddr);
int multiboot2_set_boot_mode(int fd, uint32_t zone_id,
                             __u64 multiboot_info_paddr);

#endif /* __MULTIBOOT2_H */
