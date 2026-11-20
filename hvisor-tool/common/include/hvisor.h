// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Guowei Li <2401213322@stu.pku.edu.cn>
 */
#ifndef __HVISOR_H
#define __HVISOR_H
#include <linux/ioctl.h>
#include <linux/types.h>

#include "def.h"
#include "zone_config.h"

#if CONFIG_MAGIC_VERSION != 7
#error "hvisor-tool requires hvisor ABI version 7"
#endif

#define HVISOR_DEVICE "/dev/hvisor"

#define MMAP_SIZE 4096
#define MAX_ZONES MAX_CPUS

#define SIGHVI 10

typedef HvZoneListArgs zone_list_args_t;

#define device_req HvisorDeviceReq
#define device_res HvisorDeviceRes
#define hvisor_load_image_args HvLoadImageArgs
#define hvisor_irq_line_args HvIrqLineArgs

#define HVISOR_HC_INIT_VIRTIO HvVirtioInit
#define HVISOR_HC_FINISH_REQ HvVirtioInjectIrq
#define HVISOR_HC_START_ZONE HvZoneStart
#define HVISOR_HC_SHUTDOWN_ZONE HvZoneShutdown
#define HVISOR_HC_ZONE_LIST HvZoneList
#define HVISOR_HC_CONFIG_CHECK HvConfigCheck
#define HVISOR_HC_SET_BOOT_MODE HvZoneSetBootMode
#define HVISOR_HC_GET_ECAM_BASE HvGetEcamBase
#define HVISOR_HC_GET_VIRTIO_IRQ HvVirtioGetIrq

#define hvisor_scmi_clock_args HvScmiClockArgs
#define hvisor_scmi_reset_args HvScmiResetArgs
#define hvisor_scmi_power_args HvScmiPowerArgs

#if defined(LOONGARCH64) || defined(X86_64)
#define HVISOR_HC_CLEAR_INJECT_IRQ HvClearInjectIrq
#endif

#ifdef LOONGARCH64
#define HVISOR_VIRTIO_WAKEUP_HWI 4 // LoongArch CPUINTC HWI2

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
