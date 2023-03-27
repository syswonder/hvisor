ARCH ?= aarch64
VENDOR ?= 
LOG ?=
STATS ?= off
PORT ?= 2333

# do not support debug mode
MODE := release

export MODE
export LOG
export ARCH
export VENDOR
export STATS


features :=

ifeq ($(STATS), on)
  features += --features stats
endif

build_args := --features "$(features)" --target $(ARCH).json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

ifeq ($(MODE), release)
  build_args += --release
endif

.PHONY: qemu-aarch64
qemu-aarch64:
	cargo clean
	cargo build $(build_args)

.PHONY: start
start: qemu-aarch64
	qemu-system-aarch64 \
    -M virt \
    -m 1024M \
    -cpu cortex-a53 \
    -nographic \
    -machine virtualization=on \
    -kernel target/aarch64/release/armv8-baremetal-demo-rust

.PHONY: debug
debug: qemu-aarch64
	qemu-system-aarch64 \
    -M virt \
    -m 1024M \
    -cpu cortex-a53 \
    -nographic \
    -machine virtualization=on \
    -kernel target/aarch64/release/armv8-baremetal-demo-rust \
    -S -s
