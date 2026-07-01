# Boot Notes

## Asterinas or Linux on QEMU

QEMU's `-kernel` consumes the Linux boot protocol / multiboot v1, not multiboot2.
The reliable Asterinas path on QEMU is therefore the grub-rescue ISO (multiboot2)
booted via OVMF, where GRUB's `multiboot2` command loads the kernel — no EFI
handover needed. Key facts from bring-up:

- Build the Asterinas ISO with the default `multiboot2` protocol, not
  `--grub-boot-protocol=linux --linux-x86-legacy-boot`. Under OVMF the `linux`
  protocol triggers EFI handover, which the legacy bzImage lacks
  (`error: kernel doesn't support EFI handover`).

  ```sh
  cargo osdk build --initramfs <path> --grub-boot-protocol=multiboot2 \
    --strip-elf --output _build/osdk-mb2
  ```

- QEMU 6.2 cannot boot current Asterinas images directly. Use QEMU 9.x or newer
  for direct Asterinas boots.

- Boot with a CPU model QEMU supports; `-cpu Icelake-Server` works.

- A stock Linux bzImage boots directly via `-kernel` (Linux protocol) on the `pc`
  machine with `console=ttyS0,115200` (the explicit baud is required for serial).

`harness/boot_collect.py` automates collection. Use ISO mode for Asterinas and
`--mode linux` for Linux bzImage boots. The collector captures the serial
console, decodes the hex-framed `abi_runner` JSON, and writes
`results/<env>/results.json`.

## Asterinas as hvisor zone0

hvisor's x86 root zone uses the Linux boot protocol (zeropage / boot_params). The
Asterinas OSDK bzImage carries its own setup/boot-params and is loaded as one
multiboot2 module; the initramfs is a second module that GRUB transparently
gunzips. Key GPAs: setup / boot-params `0xf_f000`, protected-mode entry
`0x10_0000`, initramfs `0x1530_0000`. The OSDK image header reports boot protocol
`0x020f`; the hvisor code only requires `>= 0x0204`. Asterinas reaches PCI config
space through ECAM at the firmware MCFG base `0xb000_0000`.

The root-zone boot config and the hvisor changes that make the boot work (CPUID
leaf `0x15`, MMIO REX.B/REX.W decoding, ECAM base, BAR window mapping) are gated
behind the `asterinas` Kconfig option. Booting under QEMU with KVM requires the
`intel-iommu` device and Intel VT-x with nested VMX. See
`docs/hvisor_zone0_boot.md` for the memory layout and boot command.

## Zone JSON templates

The Asterinas zone0 JSON templates live under `configs/`: a first-light template
(`console=ttyS0`) and an interactive virtio-console template (`hvc0`). Constraints:
command line under `arch_config.cmdline` (no top-level key); top-level `interrupts`
empty for x86; virtio devices must match `configs/virtio_cfg.json` and the
`virtio_mmio.device=<len>@<addr>:<irq>` cmdline entries.
