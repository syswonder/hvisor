#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors
"""Validate hvisor x86 zone JSON files used by the SMP/RT probe runs.

The checks here are derived directly from hvisor's x86 zone loader so that the
configs can be checked without booting hvisor/KVM:

  * src/arch/x86_64/zone.rs   -> HvArchZoneConfig field meaning, virtio MMIO is
                                 trapped at `physical_start`.
  * src/arch/x86_64/boot.rs   -> e820 / RSDP / ACPI / UEFI region-id semantics,
                                 ramdisk_image is taken from `initrd_load_gpa`.
  * src/arch/x86_64/acpi.rs   -> RSDP/ACPI tables copied into the region whose
                                 *index* equals rsdp_/acpi_memory_region_id.
  * tools/hvisor.c            -> initrd_filepath/initrd_load_hpa/initrd_load_gpa
                                 are the JSON keys that load an initrd.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

VIRTIO_RE = re.compile(r"virtio_mmio\.device=(0x[0-9a-fA-F]+|\d+)@(0x[0-9a-fA-F]+|\d+):(\d+)")

MEM_RAM = "ram"
MEM_IO = "io"
MEM_VIRTIO = "virtio"
MEM_RESERVED = "reserved"


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def parse_int(value: str | int) -> int:
    if isinstance(value, int):
        return value
    return int(value, 0)


def hpa_range(region: dict[str, Any]) -> tuple[int, int, str]:
    start = parse_int(region["physical_start"])
    size = parse_int(region["size"])
    return start, start + size, region["type"]


def gpa_range(region: dict[str, Any]) -> tuple[int, int]:
    start = parse_int(region["virtual_start"])
    size = parse_int(region["size"])
    return start, start + size


def find_ram_region_for_gpa(regions: list[dict[str, Any]], gpa: int, end: int):
    for region in regions:
        if region["type"] != MEM_RAM:
            continue
        g0, g1 = gpa_range(region)
        if g0 <= gpa and end <= g1:
            return region
    return None


def validate_zone(path: Path, errors: list[str]) -> dict[str, Any]:
    data = load_json(path)
    for key in ["arch", "zone_id", "cpus", "memory_regions", "interrupts", "arch_config", "pci_config"]:
        if key not in data:
            errors.append(f"{path}: missing {key}")
    if data.get("arch") != "x86_64":
        errors.append(f"{path}: arch must be x86_64")
    if data.get("dtb_filepath") != "":
        errors.append(f"{path}: x86 zone dtb_filepath must be empty")
    if data.get("interrupts") != []:
        errors.append(f"{path}: top-level interrupts must be [] for this x86 APIC/IOAPIC setup")
    cpus = data.get("cpus", [])
    if not isinstance(cpus, list) or not cpus or len(cpus) != len(set(cpus)):
        errors.append(f"{path}: cpus must be a non-empty unique list")

    regions = data.get("memory_regions", [])
    # No overlapping host-physical ranges among backed regions (virtio MMIO is a
    # trap region, not RAM-backed, so it is allowed to be checked too: it must
    # still not overlap real RAM).
    for idx, left in enumerate(regions):
        l0, l1, _ = hpa_range(left)
        for right in regions[idx + 1:]:
            r0, r1, _ = hpa_range(right)
            if max(l0, r0) < min(l1, r1):
                errors.append(f"{path}: overlapping HPA regions {left} and {right}")

    arch = data.get("arch_config", {})
    required_arch = [
        "ioapic_base", "ioapic_size", "boot_filepath", "boot_load_paddr",
        "cmdline", "cmdline_load_hpa", "cmdline_load_gpa", "kernel_entry_gpa",
        "setup_filepath", "setup_load_hpa", "setup_load_gpa",
        "rsdp_memory_region_id", "acpi_memory_region_id", "uefi_memory_region_id",
    ]
    for key in required_arch:
        if key not in arch:
            errors.append(f"{path}: arch_config missing {key}")

    # RSDP/ACPI/UEFI region ids are *indices* into memory_regions and the loader
    # writes RAM-backed tables there, so they must be in-bounds and type ram.
    for key, want_gpa in (("rsdp_memory_region_id", 0xE0000),
                          ("acpi_memory_region_id", None),
                          ("uefi_memory_region_id", None)):
        if key not in arch:
            continue
        rid = parse_int(arch[key])
        if rid < 0 or rid >= len(regions):
            errors.append(f"{path}: {key}={rid} is out of range (0..{len(regions) - 1})")
            continue
        if regions[rid]["type"] != MEM_RAM:
            errors.append(f"{path}: {key}={rid} must point at a ram region, got {regions[rid]['type']}")
        if want_gpa is not None and parse_int(regions[rid]["virtual_start"]) != want_gpa:
            errors.append(
                f"{path}: {key} region gpa should be {hex(want_gpa)} per hvisor, "
                f"got {regions[rid]['virtual_start']}"
            )

    # setup must live in the low 16-bit segment area.
    if "setup_load_gpa" in arch and parse_int(arch["setup_load_gpa"]) >= 0x10000:
        errors.append(f"{path}: setup_load_gpa must be < 0x10000 (16-bit), got {arch['setup_load_gpa']}")

    # kernel_entry_gpa must be backed by RAM.
    if "kernel_entry_gpa" in arch:
        keg = parse_int(arch["kernel_entry_gpa"])
        if find_ram_region_for_gpa(regions, keg, keg + 1) is None:
            errors.append(f"{path}: kernel_entry_gpa {arch['kernel_entry_gpa']} is not inside a ram region")

    # initrd validation: if requested, it must land in a RAM region (Asterinas'
    # legacy setup asserts the ramdisk is inside E820 usable RAM) and the
    # gpa->hpa offset must match the backing region's offset mapping.
    if "initrd_filepath" in arch and arch["initrd_filepath"]:
        for k in ("initrd_load_hpa", "initrd_load_gpa"):
            if k not in arch:
                errors.append(f"{path}: arch_config missing {k} (required with initrd_filepath)")
        if "initrd_load_gpa" in arch and "initrd_load_hpa" in arch:
            igpa = parse_int(arch["initrd_load_gpa"])
            ihpa = parse_int(arch["initrd_load_hpa"])
            region = find_ram_region_for_gpa(regions, igpa, igpa + 1)
            if region is None:
                errors.append(f"{path}: initrd_load_gpa {arch['initrd_load_gpa']} is not inside a ram region")
            else:
                expect_hpa = igpa - parse_int(region["virtual_start"]) + parse_int(region["physical_start"])
                if expect_hpa != ihpa:
                    errors.append(
                        f"{path}: initrd_load_hpa {hex(ihpa)} != offset-mapped {hex(expect_hpa)} "
                        f"for region gpa {region['virtual_start']} hpa {region['physical_start']}"
                    )

    # pci_config[0] is dereferenced unconditionally by boot.rs (ECAM e820 entry).
    if not data.get("pci_config"):
        errors.append(f"{path}: pci_config must have at least one entry (boot.rs reads pci_config[0])")

    return data


def validate_virtio(
    zone: dict[str, Any],
    virtio_cfg: dict[str, Any],
    zone_path: Path,
    errors: list[str],
    warnings: list[str],
) -> None:
    if zone.get("zone_id") != 1:
        return

    cmdline = zone.get("arch_config", {}).get("cmdline", "")
    cmdline_devs = {(int(length, 0), int(addr, 0), int(irq)) for length, addr, irq in VIRTIO_RE.findall(cmdline)}

    virtio_regions = [r for r in zone["memory_regions"] if r["type"] == MEM_VIRTIO]
    region = virtio_regions[0] if virtio_regions else None

    # hvisor traps virtio MMIO at the region's PHYSICAL start (zone.rs), so the
    # guest-visible virtio address equals physical_start (+offset), and this is
    # also the `addr` carried in virtio_cfg.json.
    if region is not None:
        p0, p1, _ = hpa_range(region)
        for _, addr, _ in cmdline_devs:
            if not (p0 <= addr < p1):
                errors.append(
                    f"{zone_path}: cmdline virtio addr {hex(addr)} outside virtio region "
                    f"physical range [{hex(p0)},{hex(p1)})"
                )

    zones = {entry["id"]: entry for entry in virtio_cfg.get("zones", [])}
    if 1 not in zones:
        errors.append("virtio_cfg.json: missing zone id 1")
        return
    cfg_devs = {(parse_int(d["len"]), parse_int(d["addr"]), int(d["irq"])) for d in zones[1].get("devices", [])}

    # If virtio_cfg.json advertises devices but the cmdline carries no
    # virtio_mmio.device= entries, the guest cannot discover them: warn.
    if cfg_devs and not cmdline_devs:
        warnings.append(
            f"{zone_path}: virtio_cfg.json lists {len(cfg_devs)} device(s) but cmdline has no "
            f"virtio_mmio.device= entries"
        )

    # virtio_cfg device addresses must fall in the zone's virtio trap window.
    if region is not None:
        p0, p1, _ = hpa_range(region)
        for _, addr, _ in cfg_devs:
            if not (p0 <= addr < p1):
                errors.append(
                    f"virtio_cfg.json: device addr {hex(addr)} outside zone1 virtio region "
                    f"[{hex(p0)},{hex(p1)})"
                )

    # If the cmdline advertises virtio devices, they must match virtio_cfg.json.
    if cmdline_devs and cmdline_devs != cfg_devs:
        errors.append(
            f"{zone_path}: cmdline virtio devices {sorted(cmdline_devs)} != "
            f"virtio_cfg.json devices {sorted(cfg_devs)}"
        )


def validate_cross_zone(
    zones: list[tuple[Path, dict[str, Any]]], errors: list[str], warnings: list[str]
) -> None:
    """Check every pair of distinct-zone_id configs for pCPU / HPA overlap.

    Configs sharing a zone_id are mutually-exclusive alternatives (e.g.
    zone1_aster_{1,2,4}c.json, or the two noisy variants), never co-resident, so
    they are skipped. A shared host-physical RAM range between two zones is a hard
    error (memory corruption). A shared pCPU is only a problem if the two configs
    are actually launched together: the staircase configs deliberately reuse
    pCPUs across run modes (4c uses [2,3,4,5], the matched noisy zone uses
    [6,7]), so an overlap is reported as a warning naming the configs that must
    not be co-resident rather than failing validation.
    """
    for idx, (left_path, left) in enumerate(zones):
        left_cpus = set(left.get("cpus", []))
        left_ranges = [hpa_range(r) for r in left.get("memory_regions", []) if r["type"] not in (MEM_IO,)]
        for right_path, right in zones[idx + 1:]:
            if left.get("zone_id") == right.get("zone_id"):
                continue
            overlap = left_cpus & set(right.get("cpus", []))
            if overlap:
                warnings.append(
                    f"{left_path.name} and {right_path.name} share pCPUs {sorted(overlap)};"
                    " do not run them together (see run_smp_matrix.sh for valid pairings)"
                )
            right_ranges = [hpa_range(r) for r in right.get("memory_regions", []) if r["type"] not in (MEM_IO,)]
            for l0, l1, ltype in left_ranges:
                for r0, r1, rtype in right_ranges:
                    if max(l0, r0) < min(l1, r1):
                        errors.append(
                            f"{left_path} and {right_path}: overlapping non-IO HPA ranges "
                            f"0x{l0:x}-0x{l1:x}({ltype}) / 0x{r0:x}-0x{r1:x}({rtype})"
                        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configs-dir", type=Path, default=Path("configs"))
    parser.add_argument(
        "--matrix",
        action="store_true",
        help="accepted for compatibility; the cross-zone check now runs unconditionally",
    )
    args = parser.parse_args()

    errors: list[str] = []
    warnings: list[str] = []
    virtio_cfg = load_json(args.configs_dir / "virtio_cfg.json")
    zone_paths = sorted(args.configs_dir.glob("zone*.json"))
    zones = [(path, validate_zone(path, errors)) for path in zone_paths]
    for path, zone in zones:
        validate_virtio(zone, virtio_cfg, path, errors, warnings)

    # Cross-zone isolation is checked for *every* pair of concurrent (distinct
    # zone_id) configs, so an accidentally overlapping pCPU/HPA pairing is caught
    # regardless of which configs a runner happens to start together.
    validate_cross_zone(zones, errors, warnings)

    for warning in warnings:
        print(f"WARN: {warning}", file=sys.stderr)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"validated {len(zone_paths)} zone configs and virtio_cfg.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
