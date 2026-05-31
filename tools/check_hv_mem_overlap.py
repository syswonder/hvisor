#!/usr/bin/env python3
"""
Post-link overlap check for hvisor.

Verifies that no MEM_TYPE_RAM region in ROOT_ZONE_MEMORY_REGIONS overlaps
with hvisor's own physical memory range [skernel, __hv_end).

Without this check, the root zone's Linux may allocate and write to physical
pages that hvisor uses for its page tables and heap (memory stomping),
causing "unhandled MMIO fault" panics when those corrupted page tables
are traversed.

Usage:
    python3 tools/check_hv_mem_overlap.py <ELF> <BOARD_RS>

Returns exit code 0 if no overlap, 1 if overlap detected or error.
"""

import re
import subprocess
import sys


def get_symbol_value(elf_path: str, symbol: str) -> int | None:
    """Read a symbol value from the ELF using rust-nm."""
    result = subprocess.run(
        ["rust-nm", elf_path],
        capture_output=True,
        text=True,
    )
    for line in result.stdout.splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[2] == symbol:
            return int(parts[0], 16)
    return None


def strip_rust_comments(text: str) -> str:
    """Strip Rust line comments (//) and block comments (/* ... */) from text."""
    # Strip block comments first (non-greedy)
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    # Strip line comments
    text = re.sub(r"//[^\n]*", "", text)
    return text


def parse_root_zone_memory_regions(board_rs_path: str) -> list:
    """Parse ROOT_ZONE_MEMORY_REGIONS from board.rs.

    Returns list of (mem_type, physical_start, size) tuples.
    """
    with open(board_rs_path) as f:
        content = f.read()

    # Strip comments before parsing
    content = strip_rust_comments(content)

    # Find the ROOT_ZONE_MEMORY_REGIONS array
    match = re.search(
        r"pub\s+const\s+ROOT_ZONE_MEMORY_REGIONS\s*:\s*&\[HvConfigMemoryRegion\]\s*=\s*&\[(.*?)\];",
        content,
        re.DOTALL,
    )
    if not match:
        print(f"WARN: ROOT_ZONE_MEMORY_REGIONS not found in {board_rs_path}, skipping check",
              file=sys.stderr)
        return []

    regions_str = match.group(1)

    regions = []
    # Match each HvConfigMemoryRegion block
    for block_match in re.finditer(
        r"HvConfigMemoryRegion\s*\{(.*?)\}", regions_str, re.DOTALL
    ):
        block = block_match.group(1)

        mem_type_m = re.search(r"mem_type\s*:\s*(MEM_TYPE_\w+)", block)
        pstart_m = re.search(r"physical_start\s*:\s*(0x[0-9a-fA-F]+)", block)
        size_m = re.search(r"size\s*:\s*(0x[0-9a-fA-F]+)", block)

        if mem_type_m and pstart_m and size_m:
            regions.append((
                mem_type_m.group(1),
                int(pstart_m.group(1), 16),
                int(size_m.group(1), 16),
            ))

    return regions


def do_regions_overlap(start_a: int, end_a: int, start_b: int, end_b: int) -> bool:
    """Check if [start_a, end_a) overlaps with [start_b, end_b)."""
    return start_a < end_b and start_b < end_a


def term_bold(text: str) -> str:
    """Wrap text in ANSI bold escape codes."""
    return f"\033[1m{text}\033[22m"


def term_red(text: str) -> str:
    """Wrap text in ANSI red escape codes."""
    return f"\033[31m{text}\033[39m"


def term_bold_red(text: str) -> str:
    """Wrap text in ANSI bold+red escape codes."""
    return f"\033[1;31m{text}\033[0m"


def main() -> int:
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <HVISOR_ELF> <BOARD_RS>", file=sys.stderr)
        return 1

    elf_path = sys.argv[1]
    board_rs_path = sys.argv[2]

    # Read hvisor memory range from ELF
    skernel = get_symbol_value(elf_path, "skernel")
    hv_end = get_symbol_value(elf_path, "__hv_end")

    if skernel is None:
        print(f"WARN: symbol 'skernel' not found in {elf_path}, skipping check",
              file=sys.stderr)
        return 0
    if hv_end is None:
        print(f"WARN: symbol '__hv_end' not found in {elf_path}, skipping check",
              file=sys.stderr)
        return 0

    # Parse root zone memory regions
    regions = parse_root_zone_memory_regions(board_rs_path)
    if not regions:
        return 0

    # Check each MEM_TYPE_RAM region for overlap
    found_overlap = False
    for mem_type, pstart, size in regions:
        pend = pstart + size
        if mem_type != "MEM_TYPE_RAM":
            continue
        if do_regions_overlap(pstart, pend, skernel, hv_end):
            overlap_start = max(pstart, skernel)
            overlap_end = min(pend, hv_end)
            overlap_bytes = overlap_end - overlap_start

            # --- bold/red error header ---
            print()
            print(term_bold_red("╔══════════════════════════════════════════════════════════════╗"))
            print(term_bold_red("║               HVISOR MEMORY REGION OVERLAP!                  ║"))
            print(term_bold_red("╚══════════════════════════════════════════════════════════════╝"))
            print()

            # --- what happened ---
            print(f"  hvisor physical memory: {term_bold(f'[{skernel:#x}, {hv_end:#x})')}")
            print(f"  overlaps ROOT_ZONE_MEMORY_REGIONS RAM range: "
                  f"{term_bold(f'[{pstart:#x}, {pend:#x})')}")
            print(f"  overlap: {term_bold_red(f'{overlap_bytes} bytes')} "
                  f"({term_bold(f'[{overlap_start:#x}, {overlap_end:#x})')})")
            print()

            # --- danger description ---
            print(f"  {term_bold_red('DANGER')}: Linux in the root zone treats hvisor's")
            print(f"  physical pages as free memory. The kernel page allocator will")
            print(f"  hand them out to kernel or user code and write to them,")
            print(f"  {term_bold_red('corrupting hvisor page tables')} and causing unrecoverable panics.")
            print()

            # --- fix instructions ---
            print(f"  {term_bold('Fix')}:")
            print()
            print(f"  1. Edit ROOT_ZONE_MEMORY_REGIONS in {term_bold(board_rs_path)}")
            print(f"     to exclude the hvisor range [{skernel:#x}, {hv_end:#x})")
            print(f"     from the overlapping MEM_TYPE_RAM region.")
            print()
            print(f"  2. Reduce hvisor's memory footprint to avoid overlap:")
            print(f"     - Ensure MODE=release (shrinks binary size)")
            print(f"     - Reduce HV_MEM_POOL_SIZE (src/consts.rs, currently 64MB)")
            print(f"     - Adjust BASE_ADDRESS in the linker script")
            print()

            found_overlap = True

    if found_overlap:
        return 1

    print(f"OK: hvisor [{term_bold(f'{skernel:#x}')}, {term_bold(f'{hv_end:#x}')}) "
          f"does not overlap any ROOT_ZONE_MEMORY_REGIONS RAM region")
    return 0


if __name__ == "__main__":
    sys.exit(main())
