# Asterinas-on-hvisor Linux ABI Harness

This directory contains the Linux ABI harness used to exercise Asterinas on the
`x86_64/qemu` hvisor board and on direct QEMU boots. It provides a static test
suite, an in-guest runner, initramfs packaging, serial-console collection, and
comparison tooling for A/B/C style runs:

- Env A: Asterinas on direct QEMU/KVM.
- Env B: Linux as the hvisor root zone.
- Env C: Asterinas as the hvisor root zone.

Collected JSON is written under `results/`, and generated reports are written
under `report/out/`. Both paths are ignored by git.

## Verify the harness

```sh
make verify      # builds tests + abi_runner, host smoke, lints, fixtures
```

`make verify` does not boot a guest. It builds the test suite and runner, runs a
host smoke pass, checks the Python helpers, and verifies the diff/report path with
synthetic fixture data.

## Build the initramfs

```sh
make initramfs
```

The initramfs contains the compiled tests, `abi_runner`, the autorun init script,
and any optional host-provided app binaries that are present.

## Run direct QEMU guests

For Asterinas direct-boot testing, build an Asterinas OSDK multiboot2 ISO with
this harness initramfs. Use QEMU 9.x or newer; older QEMU versions can fail to
boot current Asterinas images directly.

```sh
python3 harness/boot_collect.py --mode iso --iso <aster-osdk.iso> \
  --qemu <qemu-9.x> --accel kvm --env-id A --out results/env_a/results.json
```

For a Linux reference guest:

```sh
sh scripts/build_linux_ref.sh
python3 harness/boot_collect.py --mode linux \
  --kernel _build/linux-5.19-obj/arch/x86/boot/bzImage \
  --initrd _build/initramfs-L.cpio.gz --append 'console=ttyS0 rdinit=/init' \
  --qemu <qemu-9.x> --accel kvm --env-id ref_linux \
  --out results/ref_linux_qemu/results.json
```

## Run Asterinas as hvisor zone0

Build hvisor with the `asterinas` cargo feature, stage the Asterinas OSDK bzImage
and harness initramfs into the qemu board image, boot the hvisor ISO under QEMU
with KVM and nested VMX, and capture the framed JSON from the serial console.
The memory layout and boot command are documented in `docs/hvisor_zone0_boot.md`.

## Compare runs

```sh
make report
```

The report target reads JSON from `results/`, builds a diff matrix, and writes
Markdown, CSV, and HTML output under `report/out/`.

## Layout

- `tests/` ABI cases L1–L5 + apps, catalog, `run_all.sh`.
- `harness/` `abi_runner.c` (compiled in-guest runner), `boot_collect.py`
  (boot + capture + decode), `stability_run.py`.
- `initramfs/` `build_initramfs.sh` (+ autorun stamping), `init`.
- `diff/` `syscall_diff.py`, `classify.py` (incl. Asterinas-vs-Linux attribution).
- `report/` `gen_report.py` → md/csv/html (output under `report/out/`, git-ignored).
- `configs/` zone JSON + virtio cfg + OSDK config.
- `scripts/` reproduce / Linux-reference build / validation helpers.
- `_build/ _work/ _tools/ _toolchain/ results/ report/out/` generated data.
