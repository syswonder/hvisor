# AGENTS.md — hvisor

hvisor is a Rust-based Type-1 hypervisor supporting aarch64, riscv64, x86_64,
and loongarch64 across 20+ development boards. It runs as a `#![no_std]`
bare-metal environment at EL2 (aarch64) or HS mode (riscv64).

## Documentation

Comprehensive user and architecture documentation is available at
**[https://hvisor.syswonder.org/](https://hvisor.syswonder.org/)** (source:
[github.com/syswonder/hvisor-book](https://github.com/syswonder/hvisor-book)).
It covers board-specific setup, compilation, zone configuration, and detailed
architecture explainers.

## Quick Start

```bash
# Pick arch and board (default: ARCH=aarch64 BOARD=qemu-gicv3)
make ARCH=aarch64 BOARD=rk3588 LOG=LOG_INFO
```

Supported `ARCH`: `aarch64` `riscv64` `x86_64` `loongarch64`. Available boards
under `platform/<arch>/`.

## Source Layout

| Path | Description |
|------|-------------|
| `src/arch/<arch>/` | Arch-specific: CPU, page tables, interrupt controller, cache |
| `src/zone.rs` | Zone (virtual machine) lifecycle |
| `src/hypercall/mod.rs` | Hypercall dispatch |
| `src/device/` | Device models: irqchip, uart, iommu, virtio_trampoline |
| `src/pci/` | PCI bus emulation and passthrough |
| `src/memory/` | Memory management: frame allocator, stage-2 page tables |
| `src/config.rs` | Zone config struct definitions |
| `platform/<arch>/<board>/` | Board-specific config (board.rs + platform.mk) |

## Coding Conventions

- **Error handling**: Use `HvResult<T>` and `hv_result_err!(ENOMEM, "msg")`
  macros. Avoid bare `Err()` construction.
- **Locks**: `spin::Mutex` / `spin::RwLock` (no_std compatible). Use
  `ctrl_lock.lock()` for short critical sections.
- **unsafe**: Limit to hypercall argument dereference, MMIO register access,
  and inline assembly. Avoid elsewhere.
- **Naming**: `snake_case` for variables/functions, `UpperCamelCase` for
  types/enums.
- **Formatting**: Run `make fmt` (`cargo fmt --all`) before commit. Run
  `make fmt-test` before PR to verify formatting is clean.

## Cross-Repository Work

This repo works with
[hvisor-tool](https://github.com/syswonder/hvisor-tool), which provides:

- Kernel module `driver/hvisor.ko` — zone lifecycle, image loading, VirtIO
  communication
- Userspace binary `tools/hvisor` — zone management CLI, VirtIO device
  backends

**When the task involves any of the following, ask the user for the local
path to hvisor-tool before proceeding:**

- Hypercall ABI (`hvisor_call` in `include/def.h`, hypercall codes)
- IOCTL interface (`include/hvisor.h`)
- VirtIO protocol (shared-memory request/response in `tools/virtio/`)
- Zone config structs (`include/zone_config.h`)

## Commit Message Format

```
<type>(<scope>): <short description>

<optional: rationale>

Co-authored-by: ...
```

Types: `feat` / `fix` / `docs` / `refactor` / `ci`.
Scope: architecture or subsystem name (e.g. `aarch64`, `riscv64`, `pci`,
`virtio`).
AI attribution is optional. Consider adding it for substantial
AI-generated changes (e.g. large refactors, cross-file edits, design
work). Omit for routine edits, boilerplate, and simple fixes.
