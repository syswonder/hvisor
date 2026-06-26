<!-- SPDX-License-Identifier: MulanPSL-2.0 -->
# Static Rust Demo

`hello_rust` is optional because a Rust toolchain may not be present on the
target server. `initramfs/build_initramfs.sh` copies a prebuilt binary from
`_build/hello_rust` when it exists. Otherwise the shell smoke test reports
`SKIP` instead of pretending a Rust workload passed.
