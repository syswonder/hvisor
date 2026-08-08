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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>

#include "json_parse.h"
#include "loader.h"
#include "log.h"
#include "multiboot2.h"
#include "safe_cjson.h"

int multiboot2_enabled(struct cJSON *root) {
    cJSON *multiboot_json = cJSON_GetObjectItem(root, "multiboot_enabled");
    if (multiboot_json != NULL && cJSON_IsBool(multiboot_json)) {
        return cJSON_IsTrue(multiboot_json) ? 1 : 0;
    }
    return 0;
}

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

int multiboot2_prepare_zone(zone_config_t *config, struct cJSON *root,
                            __u64 *multiboot_info_paddr) {
#ifdef X86_64
    int64_t gpa_to_hpa_offset = 0;
    for (uint32_t i = 0; i < config->num_memory_regions; i++) {
        memory_region_t *mem_region = &config->memory_regions[i];
        if (mem_region->type == MEM_TYPE_RAM) {
            gpa_to_hpa_offset = (int64_t)mem_region->physical_start -
                                (int64_t)mem_region->virtual_start;
            break;
        }
    }

    cJSON *kernel_filepath_json = cJSON_GetObjectItem(root, "kernel_filepath");
    if (kernel_filepath_json == NULL ||
        kernel_filepath_json->valuestring == NULL) {
        log_error("[MULTIBOOT2] Missing kernel_filepath");
        return -1;
    }

    uint64_t elf_entry = 0;
    uint64_t total_size = 0;
    if (load_elf_kernel(kernel_filepath_json->valuestring, &elf_entry,
                        &total_size, gpa_to_hpa_offset) != 0) {
        log_error("Failed to load ELF segments for Multiboot kernel\n");
        return -1;
    }
    config->kernel_size = total_size;
    config->arch_config.kernel_entry_gpa = elf_entry;

    cJSON *kcmdline_json = cJSON_GetObjectItem(root, "kernel_cmdline");
    const char *cmdline = "";
    if (kcmdline_json != NULL && kcmdline_json->valuestring != NULL) {
        cmdline = kcmdline_json->valuestring;
    }

    cJSON *mb_info_paddr_json =
        cJSON_GetObjectItem(root, "multiboot_info_paddr");
    *multiboot_info_paddr = 0x9000000; // Default GPA
    if (mb_info_paddr_json != NULL &&
        parse_json_linux_u64(mb_info_paddr_json, multiboot_info_paddr) != 0) {
        log_error("Failed to parse multiboot_info_paddr\n");
        return -1;
    }

    cJSON *arch_config_json = cJSON_GetObjectItem(root, "arch_config");
    cJSON *boot_filepath_json =
        cJSON_GetObjectItem(arch_config_json, "boot_filepath");
    cJSON *boot_load_paddr_json =
        cJSON_GetObjectItem(arch_config_json, "boot_load_paddr");
    if (boot_filepath_json != NULL && boot_filepath_json->valuestring != NULL &&
        boot_load_paddr_json != NULL) {
        __u64 boot_load_paddr = 0;
        if (parse_json_linux_u64(boot_load_paddr_json, &boot_load_paddr) != 0) {
            log_error("Failed to parse boot_load_paddr\n");
            return -1;
        }
        __u64 boot_load_hpa =
            (__u64)((int64_t)boot_load_paddr + gpa_to_hpa_offset);
        load_image_to_memory(boot_filepath_json->valuestring, boot_load_hpa);
    }

    cJSON *cmdline_load_gpa_json =
        cJSON_GetObjectItem(arch_config_json, "cmdline_load_gpa");
    __u64 cmdline_gpa = *multiboot_info_paddr;
    if (cmdline_load_gpa_json != NULL &&
        parse_json_linux_u64(cmdline_load_gpa_json, &cmdline_gpa) != 0) {
        log_error("Failed to parse cmdline_load_gpa\n");
        return -1;
    }
    if (cmdline[0] != '\0') {
        config->arch_config.cmdline_load_gpa = cmdline_gpa;
        uint64_t cmdline_hpa =
            (uint64_t)((int64_t)cmdline_gpa + gpa_to_hpa_offset);
        load_buffer_to_memory(cmdline, strlen(cmdline) + 1, cmdline_hpa);
    }

    __u64 initramfs_gpa = 0;
    uint64_t initramfs_size = 0;
    cJSON *initramfs_filepath_json =
        cJSON_GetObjectItem(root, "initramfs_filepath");
    cJSON *initramfs_load_gpa_json =
        cJSON_GetObjectItem(root, "initramfs_load_gpa");
    if (initramfs_filepath_json != NULL && initramfs_load_gpa_json != NULL &&
        initramfs_filepath_json->valuestring != NULL &&
        strcmp(initramfs_filepath_json->valuestring, "null") != 0) {
        if (parse_json_linux_u64(initramfs_load_gpa_json, &initramfs_gpa) !=
            0) {
            log_error("Failed to parse initramfs_load_gpa\n");
            return -1;
        }
        uint64_t initramfs_hpa =
            (uint64_t)((int64_t)initramfs_gpa + gpa_to_hpa_offset);
        initramfs_size = load_image_to_memory(
            initramfs_filepath_json->valuestring, initramfs_hpa);
        config->arch_config.initrd_load_gpa = initramfs_gpa;
        config->arch_config.initrd_size = initramfs_size;
    }

    return 0;
#else
    (void)config;
    (void)root;
    (void)multiboot_info_paddr;
    log_error("Multiboot2 is only supported on x86_64");
    return -1;
#endif
}

int multiboot2_set_boot_mode(int fd, uint32_t zone_id,
                             __u64 multiboot_info_paddr) {
#ifdef X86_64
    struct hv_zone_boot_mode boot_mode = {
        .zone_id = zone_id,
        .multiboot_enabled = 1,
        .multiboot_info_paddr = multiboot_info_paddr,
    };
    if (ioctl(fd, HVISOR_SET_BOOT_MODE, &boot_mode)) {
        perror("zone_start: set boot mode failed");
        return -1;
    }
    return 0;
#else
    (void)fd;
    (void)zone_id;
    (void)multiboot_info_paddr;
    return -1;
#endif
}
