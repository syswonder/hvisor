#include "hvisor_abi.h"

#include <stddef.h>

_Static_assert(sizeof(HvZoneConfigCommon) == 3176, "HvZoneConfigCommon size");
_Static_assert(sizeof(Aarch64ZoneConfig) == 3296, "Aarch64ZoneConfig size");
_Static_assert(offsetof(Aarch64ZoneConfig, arch_config) == 3176,
               "Aarch64ZoneConfig arch_config offset");
_Static_assert(sizeof(Riscv64ZoneConfig) == 3208, "Riscv64ZoneConfig size");
_Static_assert(sizeof(Loongarch64ZoneConfig) == 3184,
               "Loongarch64ZoneConfig size");
_Static_assert(sizeof(X86_64ZoneConfig) == 3264, "X86_64ZoneConfig size");
_Static_assert(sizeof(HvPciDevConfig) == 12, "HvPciDevConfig size");
_Static_assert(sizeof(IvcInfo) == 56, "IvcInfo size");
_Static_assert(HvVirtioGetIrq == 86, "hypercall value");
_Static_assert(sizeof(ZoneInfo) == 56, "ZoneInfo size");
_Static_assert(sizeof(HvZoneListArgs) == 16, "HvZoneListArgs size");
_Static_assert(sizeof(HvLoadImageArgs) == 24, "HvLoadImageArgs size");
_Static_assert(sizeof(HvIrqLineArgs) == 8, "HvIrqLineArgs size");
_Static_assert(sizeof(HvIvcUserInfo) == 12, "HvIvcUserInfo size");
_Static_assert(sizeof(HvIvcControlTable) == 24, "HvIvcControlTable size");
_Static_assert(sizeof(HvVirtioBridge) > 0, "HvVirtioBridge size");
_Static_assert(sizeof(HvScmiClockArgs) == 88, "HvScmiClockArgs size");
_Static_assert(sizeof(HvScmiResetArgs) == 20, "HvScmiResetArgs size");
_Static_assert(sizeof(HvScmiPowerArgs) == 80, "HvScmiPowerArgs size");
_Static_assert(HVISOR_SCMI_CLOCK_IOCTL == 0xC0580120, "clock ioctl");
_Static_assert(HVISOR_SCMI_RESET_IOCTL == 0xC0140121, "reset ioctl");
_Static_assert(HVISOR_SCMI_POWER_IOCTL == 0xC0500122, "power ioctl");
_Static_assert(HVISOR_DEASSERT_IRQ == 0x4008010b, "deassert irq ioctl");

int main(void) { return 0; }
