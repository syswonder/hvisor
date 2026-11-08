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

int main(void) { return 0; }
