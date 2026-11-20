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
#ifndef __HVISOR_TOOL_LOADER_H
#define __HVISOR_TOOL_LOADER_H

#include <stdint.h>

#include "hvisor.h"

void *read_file(const char *filename, uint64_t *filesize);
__u64 load_buffer_to_memory(const void *buf, __u64 size, __u64 load_paddr);
__u64 load_image_to_memory(const char *path, __u64 load_paddr);

#endif /* __HVISOR_TOOL_LOADER_H */
