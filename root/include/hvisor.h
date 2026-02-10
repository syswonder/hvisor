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

#define HVISOR_HC_INIT_VIRTIO 0
#define HVISOR_HC_FINISH_REQ 1
#define HVISOR_HC_START_ZONE 2
#define HVISOR_HC_SHUTDOWN_ZONE 3
#define HVISOR_HC_ZONE_LIST 4
#define HVISOR_HC_CONFIG_CHECK 6

#ifdef X86_64

#define HVISOR_HC_GET_VIRTIO_IRQ 86

#endif /* X86_64 */

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
