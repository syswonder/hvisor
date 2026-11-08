# hvisor ABI

This directory is the single source of truth for the hvisor ABI shared by:

- the hvisor hypervisor;
- the hvisor kernel module;
- hvisor-tool, including the C implementation.

## Layout

    abi/src/config/
      common.rs      common leaf structures, constants, HvZoneConfigCommon
      pci.rs         PCI device assignment ABI
      zone.rs        impl_zone_config! constructor/accessors
      aarch64.rs     AArch64 arch config + Aarch64ZoneConfig
      riscv64.rs     RISC-V arch config + Riscv64ZoneConfig
      loongarch64.rs LoongArch arch config + Loongarch64ZoneConfig
      x86_64.rs      x86_64 arch config + X86_64ZoneConfig

The architecture-independent fields live in HvZoneConfigCommon. The
concrete ZoneConfig wrappers contain exactly:

    HvZoneConfigCommon common;
    <arch>ArchConfig    arch_config;

This keeps common and architecture-specific ABI data separate while still
producing concrete types that cbindgen can export to C.

## Generate the C header

    cbindgen --config abi/cbindgen.toml --crate hvisor-abi \
        --output abi/include/hvisor_abi.h
    cc -std=c11 -Iabi/include -c abi/tests/abi.c -o /tmp/abi.o

The generated header and the C static assertions are checked in.

## Check

    ./abi/check.sh

## Rules

- keep the crate no_std and dependency-free outside core/alloc;
- use fixed-width integers for new ABI fields;
- keep field order stable and bump CONFIG_MAGIC_VERSION when changing layout;
- add a layout test for every new structure;
- regenerate the C header after ABI changes;
- do not hand-write a second C header.

## Not extracted yet

- ioctl IDs;
- virtio controller/volatile layout is still owned by hvisor; the plain shared-memory mirror is exported by abi.
- adding CI checks for the generated C header;
- adding C layout assertions for every exported structure.
