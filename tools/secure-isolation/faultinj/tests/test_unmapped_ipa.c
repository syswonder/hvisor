/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "fault_probe.h"

int main(int argc, char **argv)
{
    uintptr_t addr = fault_probe_parse_addr(argc, argv, 0x20000000UL);
    return fault_probe_read_addr(addr, "unmapped IPA");
}
