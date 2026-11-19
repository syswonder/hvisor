// SPDX-License-Identifier: GPL-2.0-only
#ifndef __ZONE_CONFIG_H
#define __ZONE_CONFIG_H

#include "hvisor_abi.h"

typedef HvConfigMemoryRegion memory_region_t;
typedef HvPciDevConfig hv_pci_dev_config_t;
typedef HvPciConfig pci_config_t;
typedef HvIvcConfig ivc_config_t;
typedef ZoneInfo zone_info_t;

#define hv_zone_boot_mode HvZoneBootMode
#define Uefi HvUefiConfig

#if defined(ARM64)
typedef Aarch64ZoneConfig zone_config_t;
typedef Aarch64ArchConfig arch_zone_config_t;
#elif defined(RISCV64)
typedef Riscv64ZoneConfig zone_config_t;
typedef Riscv64ArchConfig arch_zone_config_t;
#elif defined(LOONGARCH64)
typedef Loongarch64ZoneConfig zone_config_t;
typedef Loongarch64ArchConfig arch_zone_config_t;
#elif defined(X86_64)
typedef X86_64ZoneConfig zone_config_t;
typedef X86_64ArchConfig arch_zone_config_t;
#else
#error "unsupported architecture"
#endif

#endif
