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
#include <stdlib.h>

#include "log.h"
#include "multiboot2.h"

int load_elf_kernel(const char *elf_path, uint64_t *entry_point,
                    uint64_t *total_size, int64_t gpa_to_hpa_offset) {
    uint64_t file_size;
    void *elf = read_file(elf_path, &file_size);

    if (!elf) {
        log_error("[ELF] Failed to read ELF file: %s", elf_path);
        return -1;
    }

    unsigned char *e_ident = (unsigned char *)elf;
    if (e_ident[0] != 0x7f || e_ident[1] != 'E' || e_ident[2] != 'L' ||
        e_ident[3] != 'F') {
        log_error("[ELF] Not a valid ELF file: %s", elf_path);
        free(elf);
        return -1;
    }

    if (e_ident[4] != 2) {
        log_error("[ELF] Not a 64-bit ELF file: %s", elf_path);
        free(elf);
        return -1;
    }

    *entry_point = *(uint64_t *)((char *)elf + 24);

    uint64_t e_phoff = *(uint64_t *)((char *)elf + 32);
    uint16_t e_phentsize = *(uint16_t *)((char *)elf + 54);
    uint16_t e_phnum = *(uint16_t *)((char *)elf + 56);

    uint64_t min_paddr = UINT64_MAX;
    uint64_t max_end = 0;

    for (int i = 0; i < e_phnum; i++) {
        char *phdr = (char *)elf + e_phoff + i * e_phentsize;

        uint32_t p_type = *(uint32_t *)(phdr + 0);
        uint64_t p_offset = *(uint64_t *)(phdr + 8);
        uint64_t p_paddr = *(uint64_t *)(phdr + 24);
        uint64_t p_filesz = *(uint64_t *)(phdr + 32);
        uint64_t p_memsz = *(uint64_t *)(phdr + 40);

        if (p_type != 1) { // PT_LOAD
            continue;
        }

        if (p_offset + p_filesz > file_size) {
            log_error("[ELF] Segment %d file offset+size exceeds file size", i);
            free(elf);
            return -1;
        }

        uint64_t load_paddr = (uint64_t)((int64_t)p_paddr + gpa_to_hpa_offset);

        if (p_filesz > 0) {
            load_buffer_to_memory((char *)elf + p_offset, p_filesz, load_paddr);
        }

        if (p_memsz > p_filesz) {
            uint64_t zero_fill_start = load_paddr + p_filesz;
            uint64_t zero_fill_size = p_memsz - p_filesz;
            void *zero_buf = calloc(1, zero_fill_size);
            if (!zero_buf) {
                log_error("[ELF] Failed to allocate zero buffer for segment %d",
                          i);
                free(elf);
                return -1;
            }
            load_buffer_to_memory(zero_buf, zero_fill_size, zero_fill_start);
            free(zero_buf);
        }

        if (p_paddr < min_paddr) {
            min_paddr = p_paddr;
        }
        if (p_paddr + p_memsz > max_end) {
            max_end = p_paddr + p_memsz;
        }
    }

    if (min_paddr == UINT64_MAX) {
        *total_size = 0;
    } else {
        *total_size = max_end - min_paddr;
    }
    free(elf);
    return 0;
}
