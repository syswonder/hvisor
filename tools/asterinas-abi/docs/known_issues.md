# Known Issues

## Asterinas ABI gaps

- **`t_ptrace_attach` — `PTRACE_ATTACH` returns EIO on Asterinas.** Linux 5.19
  passes the same test binary. `PTRACE_TRACEME` works, so the gap is the
  cross-process attach path.

## Environment-dependent SKIPs

- `t_ext2_rw` `SKIP`: no block device attached (`ABI_EXT2_DEVICE` unset).
- `gdb_batch` / `strace_echo` `SKIP`: they need host `gdb` / host `strace`
  (ptrace), which are not shipped in the minimal initramfs.

## hvisor zone0 boot dependencies

- Asterinas as hvisor zone0 needs QEMU/KVM with Intel VT-x and nested VMX, plus
  the `intel-iommu` device. The memory layout and boot command are in
  `docs/hvisor_zone0_boot.md`.

## Harness notes

- The guest shell (busybox ash on a young kernel) handled `sleep`,
  `$(command substitution)`, and nested background-job `wait` unreliably, so the
  original shell watchdog produced false `rc=124`/`rc=125`. Replaced by a
  compiled `harness/abi_runner` with poll-based timeouts. `run_all.sh` is
  retained for host smoke (`make smoke`).
- Asterinas forwards unknown `key=value` kernel-cmdline tokens to init as
  environment variables (not into `/proc/cmdline` argv); dotted names are not
  valid shell identifiers. Autorun is therefore stamped into the initramfs
  (`/etc/abi-autorun`, `/etc/abi-env`) at build time rather than parsed from the
  cmdline.
- L4 loopback socket tests require `lo` UP; `init` brings it up (Asterinas does
  this automatically, stock Linux does not), so the tests measure ABI, not
  network config.

## Timing data

- `duration_ms` and boot timings are diagnostic only. Do not use them as
  performance data.
