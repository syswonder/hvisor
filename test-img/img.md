Image来自yjy、cxy编译的linux内核，内核版本为5.4.0。
配套的jailhouse.ko，cell文件已经打过RVM1.5的patch，
rootfs来自jailhouse-image编译的demo-image-jailhouse-demo-qemu-arm64.ext4.img,约1.4G，其中有没有打过patch的jailhouse，所以需要进行替换。

qemu启动命令为

```sh
qemu-system-aarch64 -drive file=demo-image-jailhouse-demo-qemu-arm64.ext4.img,discard=unmap,if=none,id=disk,format=raw \
-m 1G -serial mon:stdio -netdev user,id=net,hostfwd=tcp::23333-:22 \
-kernel Image \
-append "root=/dev/vda mem=768M" -initrd demo-image-jailhouse-demo-qemu-arm64-initrd.img \
-cpu cortex-a57 -smp 16 -nographic -machine virt,gic-version=3,virtualization=on \
-device virtio-serial-device -device virtconsole,chardev=con -chardev vc,id=con -device virtio-blk-device,drive=disk \
-device virtio-net-device,netdev=net
```

启动后将配套的jailhouse.ko、config文件和需要测试的rvmarm.bin通过scp传入
