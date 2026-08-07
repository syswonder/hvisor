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

void *read_file(const char *filename, uint64_t *filesize);
__u64 load_buffer_to_memory(const void *buf, __u64 size, __u64 load_paddr);

int load_elf_kernel(const char *elf_path, uint64_t *entry_point,
                    uint64_t *total_size, int64_t gpa_to_hpa_offset);

#endif /* __MULTIBOOT2_H */
