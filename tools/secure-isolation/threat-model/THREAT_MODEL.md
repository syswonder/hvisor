# Asterinas Non-Root Guest Isolation Notes

This note describes the isolation assumptions for running Asterinas as an hvisor
non-root x86_64 guest and the hardening covered by the accompanying tools.

## Topology

```text
x86 host -> QEMU + KVM -> hvisor x86_64
  zone0  root Linux control plane and hvisor-tool VirtIO backend
  zone1  Asterinas guest
  zone2  Linux guest used for negative testing
```

CPU ownership, RAM mappings, MMIO windows, and VirtIO devices are declared in
the zone JSON files under `configs/`. The non-root zones are untrusted. zone0 and
the hvisor-tool control path are trusted.

## Trusted Components

- hvisor stage-2 EPT setup and fault handling.
- x86_64 VM-exit handling, including PIO, MSR, CPUID, IOAPIC, and MMIO paths.
- Zone lifecycle and zone memory registration.
- hvisor-tool VirtIO backend code that walks guest-supplied virtqueues.
- Static zone and VirtIO JSON consumed by the control plane.

Physical attacks, microarchitectural side channels, and a malicious zone0 are
outside this model.

## Isolation Boundaries

### Memory

Each zone owns an independent EPT root. Zone RAM and MMIO ranges are mapped from
the zone config, while VirtIO MMIO windows trap to hvisor instead of being mapped
as normal RAM. A guest access outside the mapped set reaches the EPT violation
path.

The x86_64 fault path now treats an unhandled non-root EPT violation as a zone
fault: it logs the fault, marks the offending zone failed, and parks that CPU.
The root zone remains fatal by design.

`zone_create` also rejects overlapping host-physical RAM ranges and x86_64 entry
points outside declared guest RAM. `remove_zone` releases registered ranges so a
later zone can reuse the memory after teardown.

### MMIO Emulation

Registered MMIO accesses are decoded by the x86_64 instruction emulator. An
unsupported opcode or failed instruction fetch returns an error to the EPT fault
path instead of panicking the hypervisor.

The emulator handles LA57 guests by resolving PML5 before walking the PML4. This
keeps instruction fetch translation correct when the guest enables 5-level page
tables.

### PIO, MSR, and Interrupt Routing

Non-root zones start with PIO interception enabled. The i8042 range is handled
explicitly so guest keyboard-controller probes do not escape as host faults.
Unhandled PIO and failed WRMSR emulation produce structured isolation logs.

IOAPIC redirection writes from a non-root zone are confined to the zone's CPU
set, and the virtual serial line entry is hidden from non-root guests.

### VirtIO

The zone0 VirtIO backend consumes descriptor tables and available rings supplied
by a guest. The backend patch in `hardening-patches/` adds bounds checks for:

- descriptor indices from the available ring and `next` fields;
- circular chains through a maximum walk depth;
- indirect descriptor table length and address range;
- descriptor buffer `[addr, addr + len)` containment inside zone RAM;
- NULL returns and allocation failures.

Malformed chains are dropped before device emulation sees a partially translated
`iovec`.

## Static Config Checks

`zonelint` rejects config errors before they reach hvisor or hvisor-tool:

- overlapping zone RAM, MMIO, or CPU sets;
- entry points and boot load addresses outside declared RAM;
- VirtIO device addresses outside the owning zone's VirtIO window;
- inconsistent device address, length, or IRQ values across zone JSON, VirtIO
  JSON, and the kernel command line;
- incomplete `pci_config` objects.

The Rust implementation is the default. The Python implementation is kept for
hosts without Cargo.

## Test Hooks

- `faultinj/` builds small guest binaries for unmapped IPA, cross-zone memory,
  unauthorized MMIO, and latency checks.
- `virtio-fuzzer/` emits malformed split-virtqueue layouts and can either print
  the generated rings or kick a live MMIO device from the owning guest.
- `scripts/run_hvisor_qemu.sh` boots the x86_64 QEMU board under KVM and captures
  the serial console.

Shared cache, memory bandwidth, and I/O bandwidth are not partitioned by these
changes. Use the noisy-neighbor probe to characterize that residual risk on the
target host.
