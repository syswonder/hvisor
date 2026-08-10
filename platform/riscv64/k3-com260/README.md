# SpaceMiT K3 (k3-com260)

## Software

- Linux source: <https://github.com/spacemit-com/linux-6.18/tree/k3-br-v1.0.0>
- Linux configuration: `k3_bianbu_defconfig`
- Cross-compilation toolchain: <https://www.spacemit.com/community/resources-download/Tools/Cross-compilation%20toolchain>

## Device support

- zone0 supports passthrough assignment of network and block devices.
- zone1 currently supports only virtio devices. Passthrough of physical devices
  is not yet supported because their dependency relationships are complex.

## Reproduction steps

1. Prepare `hvisor.bin`, the zone0 Linux `Image`, and `zone0.dtb` on the TFTP
   server.
2. Place the following files in the zone0 filesystem, in the same working
   directory:
   - The zone1 Linux `Image`, `zone1-linux.dtb`, and `rootfs2.ext4`.
   - The `hvisor` executable and `hvisor.ko` from hvisor-tool.
   - `configs/virtio-backend.json`, `configs/zone1-linux-virtio.json`, and
     `scripts/boot_zone1.sh` from this directory.
3. Power on the board and press `s` during U-Boot startup to stop autoboot.
   Enter the commands in [boot.txt](boot.txt) to load hvisor and the zone0
   kernel and DTB. The final `bootm` command starts hvisor, which then starts
   zone0.
4. After zone0 has booted, follow the commands in
   [scripts/boot_zone1.sh](scripts/boot_zone1.sh) to load `hvisor.ko`, start the
   virtio backend, and start zone1.
5. Run `screen /dev/pts/0` in zone0 to connect to the zone1 serial console.

> **Note:** SD-card support requires the MPXY extension introduced by SBI 3.0
> and is left for future work. To enable it, hvisor must report SBI version 3.0
> to the guest and forward MPXY calls to OpenSBI. hvisor does not currently
> provide this passthrough.
