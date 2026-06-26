#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Validate hvisor zone config invariants used by the ABI test harness."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CONFIG_DIR = ROOT / "configs"
VIRTIO_RE = re.compile(r"virtio_mmio\.device=(0x[0-9a-fA-F]+)@(0x[0-9a-fA-F]+):([0-9]+)")


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def fail(errors: list[str], path: Path, msg: str) -> None:
    errors.append(f"{path.relative_to(ROOT)}: {msg}")


def validate_zone(path: Path, virtio_devices: dict[tuple[str, int], str], errors: list[str]) -> None:
    data = load_json(path)
    if data.get("arch") != "x86_64":
        fail(errors, path, "arch must be x86_64")
    for forbidden in ["cmdline", "initramfs", "modules"]:
        if forbidden in data:
            fail(errors, path, f"forbidden top-level key {forbidden}")
    if data.get("interrupts") != []:
        fail(errors, path, "x86 zone interrupts must be []")
    for required_top in ["ivc_configs", "kernel_load_paddr", "dtb_load_paddr", "pci_config", "num_pci_devs", "alloc_pci_devs"]:
        if required_top not in data:
            fail(errors, path, f"missing top-level {required_top}")
    arch_config = data.get("arch_config")
    if not isinstance(arch_config, dict):
        fail(errors, path, "missing arch_config")
        return
    for required in ["cmdline", "boot_filepath", "setup_filepath", "kernel_entry_gpa"]:
        if required not in arch_config:
            fail(errors, path, f"missing arch_config.{required}")
    if arch_config.get("kernel_entry_gpa") != "0x100000":
        fail(errors, path, "kernel_entry_gpa should be 0x100000")
    if data.get("entry_point") != "0x8000":
        fail(errors, path, "entry_point should be 0x8000")

    cmdline = arch_config.get("cmdline", "")
    devices = {(addr.lower(), int(irq)): length.lower() for length, addr, irq in VIRTIO_RE.findall(cmdline)}
    if "hvc0" in cmdline:
        for key, expected_len in virtio_devices.items():
            if key not in devices:
                fail(errors, path, f"cmdline missing virtio device addr={key[0]} irq={key[1]}")
            elif devices[key] != expected_len:
                fail(errors, path, f"virtio len mismatch for {key}: {devices[key]} != {expected_len}")

    pci_required = {
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
    }
    for idx, pci in enumerate(data.get("pci_config", [])):
        missing = sorted(pci_required - set(pci))
        if missing:
            fail(errors, path, f"pci_config[{idx}] missing {missing}")


def main() -> int:
    errors: list[str] = []
    virtio = load_json(CONFIG_DIR / "virtio_cfg.json")
    virtio_devices: dict[tuple[str, int], str] = {}
    for zone in virtio.get("zones", []):
        for dev in zone.get("devices", []):
            virtio_devices[(dev["addr"].lower(), int(dev["irq"]))] = dev["len"].lower()
    if not virtio_devices:
        errors.append("configs/virtio_cfg.json: no virtio devices found")

    for path in sorted(CONFIG_DIR.glob("zone1_*.json")):
        validate_zone(path, virtio_devices, errors)

    if errors:
        for err in errors:
            print(err)
        return 1
    print("configs ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
