
qemu-system-aarch64 \
-drive file=./rootfs.qcow2,discard=unmap,if=none,id=disk,format=qcow2 \
-device virtio-blk-device,drive=disk \
-m 1G -serial mon:stdio  \
-kernel jail-img \
-append "root=/dev/vda mem=768M"  \
-cpu cortex-a57 -smp 16 -nographic -machine virt,gic-version=3,virtualization=on \
-device virtio-serial-device -device virtconsole,chardev=con \
-chardev vc,id=con  \
-net nic \
-net user,hostfwd=tcp::2333-:22