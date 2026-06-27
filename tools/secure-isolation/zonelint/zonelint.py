#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Fallback zonelint implementation for hosts without a Rust toolchain."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Region:
    kind: str
    physical_start: int
    virtual_start: int
    size: int


@dataclass(frozen=True)
class Zone:
    path: Path
    name: str
    zone_id: int
    cpus: list[int]
    regions: list[Region]
    entry_point: int
    kernel_load_paddr: int
    arch_config: dict
    pci_config: list[dict]


# Keys required for every pci_config entry by hvisor-tool's parse_pci_config
# (tools/hvisor.c); each is guarded by CHECK_JSON_NULL_ERR_OUT, so a config
# missing any of them fails to load on the real control plane.
PCI_CONFIG_REQUIRED_KEYS = [
    "ecam_base",
    "ecam_size",
    "io_base",
    "io_size",
    "pci_io_base",
    "mem32_base",
    "mem32_size",
    "pci_mem32_base",
    "mem64_base",
    "mem64_size",
    "pci_mem64_base",
    "bus_range_begin",
    "bus_range_end",
    "domain",
]


def parse_int(value) -> int:
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        return int(value, 0)
    raise TypeError(f"expected integer-compatible value, got {value!r}")


def load_zone(path: Path) -> Zone:
    data = json.loads(path.read_text())
    regions = [
        Region(
            kind=item["type"],
            physical_start=parse_int(item["physical_start"]),
            virtual_start=parse_int(item["virtual_start"]),
            size=parse_int(item["size"]),
        )
        for item in data["memory_regions"]
    ]
    return Zone(
        path=path,
        name=data["name"],
        zone_id=parse_int(data["zone_id"]),
        cpus=[parse_int(v) for v in data["cpus"]],
        regions=regions,
        entry_point=parse_int(data["entry_point"]),
        kernel_load_paddr=parse_int(data["kernel_load_paddr"]),
        arch_config=data["arch_config"],
        pci_config=list(data.get("pci_config", [])),
    )


def load_virtio(path: Path) -> dict:
    return json.loads(path.read_text())


def add(violations: list[str], rule: str, detail: str) -> None:
    violations.append(f"{rule}: {detail}")


def contains_region(
    regions: list[Region], kind: str, addr: int, size: int, attr: str
) -> bool:
    end = addr + size
    return any(
        r.kind == kind and getattr(r, attr) <= addr and end <= getattr(r, attr) + r.size
        for r in regions
    )


def parse_cmdline_virtio(cmdline: str) -> set[tuple[int, int, int]]:
    out: set[tuple[int, int, int]] = set()
    for token in cmdline.split():
        if not token.startswith("virtio_mmio.device="):
            continue
        spec = token.split("=", 1)[1]
        try:
            length_text, rest = spec.split("@", 1)
            addr_text, irq_text = rest.split(":", 1)
            out.add((parse_int(addr_text), parse_int(length_text), parse_int(irq_text)))
        except ValueError:
            continue
    return out


def validate(zones: list[Zone], virtio: dict | None) -> list[str]:
    violations: list[str] = []

    seen_ids: dict[int, str] = {}
    seen_cpus: dict[int, str] = {}
    for zone in zones:
        if zone.zone_id in seen_ids:
            add(
                violations,
                "zone-id-unique",
                f"zone id {zone.zone_id} reused by {seen_ids[zone.zone_id]} and {zone.name}",
            )
        seen_ids[zone.zone_id] = zone.name

        local_cpus: set[int] = set()
        for cpu in zone.cpus:
            if cpu in local_cpus:
                add(violations, "cpu-unique-within-zone", f"{zone.name} repeats CPU {cpu}")
            local_cpus.add(cpu)
            if cpu in seen_cpus:
                add(
                    violations,
                    "cpu-overlap",
                    f"CPU {cpu} assigned to both {seen_cpus[cpu]} and {zone.name}",
                )
            seen_cpus[cpu] = zone.name

        for idx, region in enumerate(zone.regions):
            if region.size <= 0:
                add(violations, "region-nonzero-size", f"{zone.name} region {idx}")

        if not contains_region(zone.regions, "ram", zone.entry_point, 1, "virtual_start"):
            add(
                violations,
                "entry-point-in-ram",
                f"{zone.name} entry_point=0x{zone.entry_point:x}",
            )
        if not contains_region(zone.regions, "ram", zone.kernel_load_paddr, 1, "physical_start"):
            add(
                violations,
                "kernel-load-in-ram",
                f"{zone.name} kernel_load_paddr=0x{zone.kernel_load_paddr:x}",
            )

        for key, physical in (
            ("boot_load_paddr", True),
            ("cmdline_load_hpa", True),
            ("setup_load_hpa", True),
            ("cmdline_load_gpa", False),
            ("setup_load_gpa", False),
            ("kernel_entry_gpa", False),
        ):
            if key not in zone.arch_config:
                continue
            value = parse_int(zone.arch_config[key])
            attr = "physical_start" if physical else "virtual_start"
            if not contains_region(zone.regions, "ram", value, 1, attr):
                add(violations, "boot-address-in-ram", f"{zone.name} {key}=0x{value:x}")

        for idx, entry in enumerate(zone.pci_config):
            for pkey in PCI_CONFIG_REQUIRED_KEYS:
                if pkey not in entry:
                    add(
                        violations,
                        "pci-config-complete",
                        f"{zone.name} pci_config[{idx}] missing required key "
                        f"'{pkey}' (hvisor-tool parse_pci_config requires all "
                        f"{len(PCI_CONFIG_REQUIRED_KEYS)} fields)",
                    )

    ram_ranges: list[tuple[int, int, str]] = []
    mmio_ranges: list[tuple[int, int, str, str]] = []
    for zone in zones:
        for region in zone.regions:
            if region.kind == "ram":
                ram_ranges.append(
                    (region.physical_start, region.physical_start + region.size, zone.name)
                )
            if region.kind in {"io", "virtio"}:
                mmio_ranges.append(
                    (
                        region.physical_start,
                        region.physical_start + region.size,
                        zone.name,
                        region.kind,
                    )
                )

    for ranges, rule in ((ram_ranges, "ram-overlap"), (mmio_ranges, "mmio-overlap")):
        ranges.sort()
        for left, right in zip(ranges, ranges[1:]):
            if left[2] != right[2] and left[1] > right[0]:
                add(violations, rule, f"{left} overlaps {right}")

    if virtio is not None:
        zones_by_id = {z.zone_id: z for z in zones}
        virtio_by_id = {parse_int(z["id"]): z for z in virtio["zones"]}
        virtio_zone0_ranges: list[tuple[int, int, int, int]] = []
        for zone in zones:
            cmdline_set = parse_cmdline_virtio(zone.arch_config.get("cmdline", ""))
            vzone = virtio_by_id.get(zone.zone_id)
            if vzone is None:
                if cmdline_set or any(r.kind == "virtio" for r in zone.regions):
                    add(violations, "virtio-zone-present", zone.name)
                continue

            cfg_set: set[tuple[int, int, int]] = set()
            for dev in vzone["devices"]:
                if dev.get("status", "enable") != "enable":
                    continue
                addr = parse_int(dev["addr"])
                length = parse_int(dev["len"])
                irq = parse_int(dev["irq"])
                cfg_set.add((addr, length, irq))
                if not contains_region(zone.regions, "virtio", addr, length, "physical_start"):
                    add(
                        violations,
                        "virtio-device-in-zone-region",
                        f"{zone.name} {dev['type']} addr=0x{addr:x} len=0x{length:x}",
                    )

            for missing in cfg_set - cmdline_set:
                add(violations, "virtio-cmdline-match", f"{zone.name} missing {missing}")
            for missing in cmdline_set - cfg_set:
                add(violations, "virtio-config-match", f"{zone.name} missing {missing}")

        for zone_id in virtio_by_id:
            if zone_id not in zones_by_id:
                add(violations, "virtio-zone-known", f"unknown zone id {zone_id}")

        for vzone in virtio["zones"]:
            zone_id = parse_int(vzone["id"])
            for idx, mem in enumerate(vzone["memory_region"]):
                start = parse_int(mem["zone0_ipa"])
                size = parse_int(mem["size"])
                if size <= 0:
                    add(violations, "virtio-memory-nonzero", f"zone {zone_id} region {idx}")
                    continue
                virtio_zone0_ranges.append((start, start + size, zone_id, idx))

        virtio_zone0_ranges.sort()
        for left, right in zip(virtio_zone0_ranges, virtio_zone0_ranges[1:]):
            if left[1] > right[0]:
                add(violations, "virtio-memory-overlap", f"{left} overlaps {right}")

    return violations


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--zone", action="append", required=True, type=Path)
    parser.add_argument("--virtio", type=Path)
    args = parser.parse_args()

    zones = [load_zone(path) for path in args.zone]
    virtio = load_virtio(args.virtio) if args.virtio else None
    violations = validate(zones, virtio)
    if violations:
        print(f"FAIL: {len(violations)} violation(s)", file=sys.stderr)
        for item in violations:
            print(f"- {item}", file=sys.stderr)
        return 1

    print(f"PASS: {len(zones)} zone config(s) validated")
    if virtio is not None:
        devices = sum(len(zone["devices"]) for zone in virtio["zones"])
        print(f"PASS: virtio config validated (zones={len(virtio['zones'])}, devices={devices})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
