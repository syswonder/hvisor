# qemu-asterinas

Boot [Asterinas](https://github.com/asterinas/asterinas) as the hvisor x86_64
root zone under QEMU/KVM.

Asterinas is a Rust OS kernel that, through its OSDK, can emit a Linux/x86
`bzImage`. hvisor already boots an x86 root zone through the Linux/x86 32-bit
boot protocol, so this board reuses that path: the `bzImage` is split into its
setup (zeropage) and protected-mode kernel, grub loads them as multiboot2
modules, and hvisor hands control to the kernel exactly as it does for a Linux
guest. No Asterinas-specific load path is added to the hypervisor.

## Boot flow

```
grub (multiboot2)
  └─ hvisor                         loaded at its multiboot2 entry
       ├─ module_init               relocates boot.bin / setup / kernel / initramfs
       ├─ BootParams::fill          writes the zeropage: e820, cmdline, ramdisk, ...
       └─ vm-entry  rip=0x8000, eax=0x100000, esi=setup_gpa
            └─ boot.bin             real mode → protected mode → jmp 0x100000
                 └─ Asterinas setup (code32_start = 0x100000, esi = boot_params)
                      └─ Asterinas kernel → OSTD → init (/init) → /bin/sh
```

The guest images are placed at the same guest-physical addresses the Linux root
zone uses:

| image                   | guest phys    | notes                                   |
| ----------------------- | ------------- | --------------------------------------- |
| `boot.bin`              | `0x8000`      | real-mode to protected-mode trampoline  |
| command line            | `0x9000`      | filled from `ROOT_ZONE_CMDLINE`         |
| `asterinas-setup.bin`   | `0xa000`      | Linux zeropage / setup header           |
| `asterinas-vmlinux.bin` | `0x100000`    | protected-mode kernel (`code32_start`)  |
| `initramfs.cpio.gz`     | `0x1530_0000` | busybox initramfs (ramdisk)             |

## Prerequisites

- The hvisor x86_64 toolchain (see the repository README).
- `build-essential` (GNU `as`/`ld`/`objdump` build the real-mode boot stub) and
  `bc` (used by the build summary).
- `qemu-system-x86_64` with KVM, and nested VMX enabled on the host
  (`/sys/module/kvm_intel/parameters/nested` is `Y`).
- `grub-mkrescue` (`grub-common` + `grub-efi-amd64-bin`), `xorriso` and
  `mtools`, plus OVMF firmware at `/usr/share/ovmf/OVMF.fd`.
- A static `busybox` (`busybox-static`), plus `cpio` and `gzip`, for the
  initramfs.
- An Asterinas v0.18.0 checkout whose `OSDK.toml` sets
  `[grub] boot_protocol = "linux"`, the matching Rust nightly with
  `rust-src`/`rustc-dev`/`llvm-tools-preview`, `cargo-osdk`, and the prebuilt
  [`linux_vdso`](https://github.com/asterinas/linux_vdso) libraries.

## Build and run

```sh
# Build the guest images.
export ASTERINAS_DIR=/path/to/asterinas
export VDSO_LIBRARY_DIR=/path/to/linux_vdso
./platform/x86_64/qemu-asterinas/build-asterinas-image.sh

# Build hvisor and run the board.
make ARCH=x86_64 BOARD=qemu-asterinas run

# Optional boot smoke check. Requires KVM and the guest images above.
./platform/x86_64/qemu-asterinas/test/boot-smoke.sh
```

`build-asterinas-image.sh` writes `asterinas-setup.bin`, `asterinas-vmlinux.bin`
and `initramfs.cpio.gz` into `image/kernel/`, which the board Makefile copies into
the ISO. The guest boots to a busybox shell on `ttyS0`.

## Notes

This board uses two x86_64 boot-path details:

- **CPUID leaf 0x15 (`cpuid.rs`, `trap.rs`).** Asterinas reads the TSC frequency
  from CPUID `0x15`; without it the kernel falls back to PIT-based calibration.
  The leaf is synthesized from the frequency hvisor already measures against the
  HPET.
- **Overlap-safe module relocation (`boot.rs`).** The setup, kernel and
  initramfs modules are relocated to fixed addresses that may overlap the region
  another not-yet-moved module still occupies. `module_init` now orders the
  copies so no module is overwritten before it is relocated.
