# Booting Asterinas as the hvisor root zone on x86_64

This guide covers the Asterinas root-zone path for the `x86_64/qemu` board. It
documents the memory layout, the hvisor changes used by this path, and the QEMU
boot command.

## Memory layout (GPA / HPA)

GRUB loads hvisor and the zone0 modules at fixed host physical addresses (HPA)
that map to the root zone's guest physical addresses (GPA). The Asterinas OSDK
bzImage is loaded as one module (it carries its own setup/boot-params); the
gunzipped initramfs is a second module.

| Object | GPA | Notes |
|--------|-----|-------|
| boot.bin trampoline | `0x8000` | zone0 entry trampoline |
| Asterinas setup / boot-params (zeropage) | `0xf_f000` | written by `BootParams::fill` |
| Asterinas protected-mode entry | `0x10_0000` | 32-bit entry; the OSDK image header reports boot protocol `0x020f`, and the hvisor code only requires `>= 0x0204` |
| initramfs (cpio) | `0x1530_0000` | GRUB transparently gunzips the `.cpio.gz`; the size bound covers the ~44 MiB decompressed cpio |
| ECAM / PCI MMCONFIG base | `0xb000_0000` | firmware MCFG base on QEMU q35; Asterinas reaches PCI config space exclusively through ECAM |

## hvisor changes used by this path

Asterinas exercises x86 paths that the hvisor `x86_64/qemu` board had not run
before. The root-zone boot config is gated behind the `asterinas` cargo feature.

1. **CPUID leaf `0x15` (TSC / crystal-clock) emulation.** Asterinas calibrates its
   timer from CPUID leaf `0x15`; hvisor previously did not answer that leaf.

2. **MMIO instruction-emulator REX.B / REX.W decoding.** Asterinas programs the
   virtual I/O APIC with `mov [r10], r9d` (REX.R + REX.B, extended registers
   `r9`/`r10`). hvisor's MMIO decoder only handled REX.R and asserted otherwise,
   computing the wrong base register; it now decodes REX.B and REX.W.

3. **ECAM (PCI MMCONFIG) base.** Asterinas reaches PCI config space exclusively
   through ECAM at the firmware MCFG base (`0xb000_0000` on QEMU q35). Linux on
   this board uses the legacy `0xcf8`/`0xcfc` ports, so the ECAM window had never
   been exercised. The ECAM base is selected by the `asterinas` cargo feature.

4. **PCI 32-bit MMIO BAR window mapped through to the zone.** Asterinas consumes
   the firmware-assigned device BARs it reads over ECAM. hvisor's lazy BAR mapping
   only runs on legacy-port BAR writes, so the 32-bit MMIO BAR window is mapped up
   front for the Asterinas zone.

5. **Root-zone boot config behind the `asterinas` cargo feature.** The OSDK
   bzImage is loaded as one module (setup / boot-params at GPA `0xf_f000`, 32-bit
   entry at `0x10_0000`; the OSDK image header reports boot protocol `0x020f`,
   while the hvisor code only requires `>= 0x0204`); the initramfs is a separate
   module at GPA `0x1530_0000` (GRUB transparently gunzips it, so the size bound
   covers the ~44 MiB decompressed cpio); the cmdline ends with `-- /init`. A GRUB
   menu entry and `platform.mk` staging complete the wiring.

## Boot pipeline

1. hvisor starts: `Hello, start HVISOR at 0xffffff8000200000!`.
2. `module_init` — hvisor processes the multiboot2 modules (GRUB has loaded
   hvisor plus the zone0 modules at this point).
3. The boot vCPU enters Asterinas: `[setup] Entering the Asterinas entry point at
   0x8001000`. The trace prints `0x8001000` as the point control transfers into
   Asterinas after the OSDK setup stub; the 32-bit protected-mode entry in the
   boot config is at GPA `0x10_0000` (see the memory layout above).
4. The I/O APIC is discovered: `INFO: irq: IOAPIC found at 0xfec00000, ID 0,
   version 17, interrupt base 0, interrupt count 23`.
5. `OSTD initialized. Preparing components.`
6. PCI is enumerated: `INFO: pci: initializing the PCI bus with bus numbers
   `0..=1``.
7. `Spawn the first kernel thread`, after which `/init` autoruns the layered ABI
   suite (`===ABI-INIT=== env=C autorun`).
8. The runner frames the result JSON on the console and prints `===ABI-DONE===`.

## Build and boot with QEMU/KVM

```sh
# 1. Build the harness initramfs from tracked sources:
(cd tools/asterinas-abi && make initramfs)

# 2. Build hvisor with the asterinas feature and the x86_64/qemu board.
#    ASTERINAS_KERNEL points at the OSDK-built Asterinas bzImage. If
#    ASTERINAS_INITRD is omitted, the build uses tools/asterinas-abi/_build/
#    initramfs.cpio.gz when it exists.
make ARCH=x86_64 BOARD=qemu \
  FEATURES="pci ecam_pcie no_pcie_bar_realloc uart16550a intel_vtd asterinas" \
  ASTERINAS_KERNEL=aster-kernel-osdk-bin

# 3. Boot the hvisor ISO under QEMU/KVM with VT-x and the VT-d IOMMU:
qemu-system-x86_64 \
  -cpu host,+vmx -accel kvm \
  -machine q35,kernel-irqchip=split \
  -smp 4 -m 4G \
  -device intel-iommu,intremap=on,eim=on,caching-mode=on,device-iotlb=on,aw-bits=48 \
  -drive file=platform/x86_64/qemu/image/virtdisk/hvisor.iso,format=raw,index=0,media=disk \
  -nographic -serial mon:stdio -no-reboot
```

Asterinas boots as zone0, runs the autorun ABI suite, and frames the results on
the console. Capture the console, decode the framed JSON to
`results/env_c/results.json`, then run `make report` if a comparison report is
needed.
