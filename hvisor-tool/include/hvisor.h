// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Guowei Li <2401213322@stu.pku.edu.cn>
 */
#ifndef __HVISOR_H
#define __HVISOR_H
#include <linux/ioctl.h>
#include <linux/types.h>

#include "def.h"
#include "zone_config.h"

#define HVISOR_DEVICE "/dev/hvisor"

#define MMAP_SIZE 4096
#define MAX_REQ 32
#define MAX_DEVS 8
#define MAX_CPUS 32
#define MAX_ZONES MAX_CPUS

#define SIGHVI 10
// receive request from el2
struct device_req {
    __u64 src_cpu;
    __u64 address; // zone's ipa
    __u64 size;
    __u64 value;
    __u32 src_zone;
    __u8 is_write;
    __u8 need_interrupt;
    __u16 padding;
};

struct device_res {
    __u32 target_zone;
    __u32 irq_id;
};

struct virtio_bridge {
    __u32 req_front;
    __u32 req_rear;
    __u32 res_front;
    __u32 res_rear;
    struct device_req req_list[MAX_REQ];
    struct device_res res_list[MAX_REQ];
    __u64 cfg_flags[MAX_CPUS]; // avoid false sharing, set cfg_flag to u64
    __u64 cfg_values[MAX_CPUS];
    // TODO: When config is okay to use, remove these. It's ok to remove.
    __u64 mmio_addrs[MAX_DEVS];
    __u8 mmio_avail;
    __u8 need_wakeup;
};

struct ioctl_zone_list_args {
    __u64 cnt;
    zone_info_t *zones;
};

typedef struct ioctl_zone_list_args zone_list_args_t;

#define HVISOR_INIT_VIRTIO _IO(1, 0) // virtio device init
#define HVISOR_GET_TASK _IO(1, 1)
#define HVISOR_FINISH_REQ _IO(1, 2) // finish one virtio req
#define HVISOR_ZONE_START _IOW(1, 3, zone_config_t *)
#define HVISOR_ZONE_SHUTDOWN _IOW(1, 4, __u64)
#define HVISOR_ZONE_LIST _IOR(1, 5, zone_list_args_t *)
#define HVISOR_CONFIG_CHECK _IOR(1, 6, __u64 *)
#define HVISOR_SET_EVENTFD _IOW(1, 7, int)
#define HVISOR_SCMI_CLOCK_IOCTL _IOWR(1, 32, struct hvisor_scmi_clock_args)
#define HVISOR_SCMI_RESET_IOCTL _IOWR(1, 33, struct hvisor_scmi_reset_args)

/* Copy user buffer to [load_paddr, load_paddr + size) in kernel. */
struct hvisor_load_image_args {
    __u64 user_buffer;
    __u64 size;
    __u64 load_paddr;
};
#define HVISOR_LOAD_IMAGE _IOW(1, 8, struct hvisor_load_image_args)

#define HVISOR_HC_INIT_VIRTIO 0
#define HVISOR_HC_FINISH_REQ 1
#define HVISOR_HC_START_ZONE 2
#define HVISOR_HC_SHUTDOWN_ZONE 3
#define HVISOR_HC_ZONE_LIST 4
#define HVISOR_HC_CONFIG_CHECK 6

#ifdef X86_64

#define HVISOR_HC_GET_VIRTIO_IRQ 86

#endif /* X86_64 */

/* SCMI Clock Subcommands */
#define HVISOR_SCMI_CLOCK_GET_COUNT 0x01
#define HVISOR_SCMI_CLOCK_GET_ATTRIBUTES 0x02
#define HVISOR_SCMI_CLOCK_DESCRIBE_RATES 0x03
#define HVISOR_SCMI_CLOCK_RATE_GET 0x04
#define HVISOR_SCMI_CLOCK_RATE_SET 0x05
#define HVISOR_SCMI_CLOCK_CONFIG_GET 0x06
#define HVISOR_SCMI_CLOCK_CONFIG_SET 0x07
#define HVISOR_SCMI_CLOCK_NAME_GET 0x08
#define HVISOR_SCMI_CLOCK_SET_PHANDLE 0xff

/* SCMI Reset Subcommands */
#define HVISOR_SCMI_RESET_RESET 0x01
#define HVISOR_SCMI_RESET_SET_PHANDLE 0xff

/* SCMI Clock ioctl argument structure */
struct hvisor_scmi_clock_args {
    __u32 subcmd;   /* Subcommand ID */
    __u32 data_len; /* Length of data buffer */
    union {
        __u32 clock_count; /* For GET_COUNT */
        struct {
            __u32 clock_id;
            __u32 enabled;
            __u32 parent_id;
            char clock_name[64];
            __u32 is_valid; /* Whether the clock is valid */
        } clock_attr;       /* For GET_ATTRIBUTES */
        struct {
            __u32 clock_id;
            __u32 rate_index;
            __u32 num_rates;
            __u32 remaining;
            __u64 rates[8]; /* Max 8 rates per response */
        } clock_rates_info; /* For DESCRIBE_RATES */
        struct {
            __u32 clock_id;
            __u64 rate;
        } clock_rate_info; /* For RATE_GET */
        struct {
            __u32 clock_id;
            __u32 flags;
            __u64 rate;
        } clock_rate_set_info; /* For RATE_SET */
        struct {
            __u32 clock_id;
            __u32 flags;
            __u32 config;
            __u32 extended_config_val;
        } clock_config_info; /* For CONFIG_GET/CONFIG_SET */
        struct {
            __u32 clock_id;
            char name[64];
        } clock_name_info; /* For NAME_GET */
        struct {
            __u32 phandle;
        } clock_phandle_info; /* For SET_PHANDLE */
        __u8 data[0];         /* For other commands */
    } u;
};

/* SCMI Reset ioctl argument structure */
struct hvisor_scmi_reset_args {
    __u32 subcmd;   /* Subcommand ID */
    __u32 data_len; /* Length of data buffer */
    union {
        struct {
            __u32 domain_id;
            __u32 flags;
            __u32 reset_state;
        } reset_info; /* For RESET */
        struct {
            __u32 phandle;
        } reset_phandle_info; /* For SET_PHANDLE */
        __u8 data[0];         /* For other commands */
    } u;
};

#ifdef LOONGARCH64

#define HVISOR_CLEAR_INJECT_IRQ _IO(1, 6) // used for ioctl
#define HVISOR_HC_CLEAR_INJECT_IRQ 20     // hvcall code in hvisor

#endif /* LOONGARCH64 */
#ifdef LOONGARCH64
static inline __u64 hvisor_call(__u64 code, __u64 arg0, __u64 arg1) {
    register __u64 a0 asm("a0") = code;
    register __u64 a1 asm("a1") = arg0;
    register __u64 a2 asm("a2") = arg1;
    // asm volatile ("hvcl"); // not supported by loongarch gcc now
    // hvcl 0 is 0x002b8000
    __asm__(".word 0x002b8000" : "+r"(a0), "+r"(a1), "+r"(a2));
    return a0;
}
#endif /* LOONGARCH64 */

#endif /* __HVISOR_H */