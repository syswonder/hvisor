/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "fault_probe.h"

int main(int argc, char **argv)
{
    uintptr_t addr = fault_probe_parse_addr(argc, argv, 0x40400000UL);
    int rc_read = fault_probe_read_addr(addr, "cross-zone memory");
    if (rc_read != 0) {
        return rc_read;
    }
    return fault_probe_write_addr(addr, "cross-zone memory");
}
