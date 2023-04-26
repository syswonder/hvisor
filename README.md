# armv8-baremetal-demo-rust

使用rust写的armv8 hypervisor，Porting from 
https://github.com/rcore-os/RVM1.5

## ~/.cargo/config
```shell
[build]
target = "aarch64-unknown-none"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "link-arg=-nostartfiles -Tlinker.ld",
]

[target.aarch64-unknown-none]
linker = "aarch64-none-elf-gcc"
```
其中需要安装linker：`aarch64-none-elf-` 地址：https://developer.arm.com/-/media/Files/downloads/gnu-a/10.3-2021.07/binrel/gcc-arm-10.3-2021.07-x86_64-aarch64-none-elf.tar.xz?rev=9d9808a2d2194b1283d6a74b40d46ada&hash=4E429A41C958483C9DB8ED84B051D010F86BA624

安装rust toolchain：`rustup install nightly && rustup default nightly && rustup target add aarch64-unknown-none (optional, we use json config)`

`apt install gdb-multiarch`

## 编译
```shell
make
```

## Qemu
```shell
make start
```
OR
```shell
qemu-system-aarch64 \
    -M virt \
    -m 1024M \
    -cpu cortex-a53 \
    -nographic \
    -kernel target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust
```
## Qemu调试
```shell
qemu-system-aarch64 \
    -M virt \
    -m 1024M \
    -cpu cortex-a53 \
    -nographic \
    -machine virtualization=on \ 
    #-machine secure=on \
    -kernel target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust \
    -S -s
```
然后使用

`gdb-multiarch target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust `

进入gdb 输入：`target remote :1234` 即开始调试
> PS: -machine virtualization=on开启虚拟化，则启用EL2，-machine secure=on，则启用EL3。我们只需要从EL2启动即可。
然后使用aarch64-linux-gnu-gdb -x debug.gdb。qemu默认从EL1启动virt

参考：
1. https://stackoverflow.com/questions/42824706/qemu-system-aarch64-entering-el1-when-emulating-a53-power-up
2. https://stackoverflow.com/questions/31787617/what-is-the-current-execution-mode-exception-level-etc
3. https://github.com/cirosantilli/linux-kernel-module-cheat/tree/35684b1b7e0a04a68987056cb15abd97e3d2f0cc#arm-exception-level

## Type 1.5 启动

1. 下载并制作ubuntu镜像，在qemu中启动
```shell
make image
```

2. 在qemu中启动制作好的虚拟机镜像
```shell
make qemu
```


## 编译GDB for aarch64（未成功）
1. 下载gdb source: https://ftp.gnu.org/gnu/gdb/gdb-13.1.tar.gz
2. tar -xzvf gdb-13.1.tar.gz
3. mkdir build 
4. cd $_
5. ../configure --prefix=$PWD --target=aarch64-linux-gnu
PS: host填写自己电脑里交叉编译器的前缀，然后configure会自动寻找该前缀的编译器
6. make -j$(nproc) [CFALGS=-static CXXFLAGS=-static]

PS: --target指定要调试的程序的架构 --host指定运行gdb程序的架构

参考：
http://ruer.fun/2021/04/20/GDB-%E7%BC%96%E8%AF%91%E6%96%B9%E6%B3%95-2021/
## 编译qemu（可用）
1. download qemu7.2.source
2. tar 解压
3. mkdir build %% cd build
4. ../qemu-7.2.0/configure --enable-kvm --enable-slirp --enable-debug --target-list=aarch64-softmmu,x86_64-softmmu
5. make -j2