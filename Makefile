ARCH ?= aarch64
LOG ?= info
STATS ?= off
PORT ?= 2333

# default debug mode
MODE ?=debug

export MODE
export LOG
export ARCH
export STATS

OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

build_path := target/$(ARCH)/$(MODE)
target_elf := $(build_path)/hvisor
target_bin := $(build_path)/hvisor.bin
guest_obj  := demo/linux-loader-linked.o
features :=

ifeq ($(STATS), on)
  features += --features stats
endif

build_args := --features "$(features)" --target $(ARCH).json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

ifeq ($(MODE), release)
  build_args += --release
endif

# .PHONY: qemu-aarch64
# qemu-aarch64:
# 	cargo clean
# 	cargo build $(build_args)

.PHONY: all
all: $(target_bin)

.PHONY: elf
elf:
	cargo build $(build_args)
.PHONY: scp
scp: $(target_bin)
	scp -P $(PORT) -r $(target_bin) qemu-test/guest/* scp root@localhost:~/
.PHONY: disa
disa:
	rust-objdump --disassemble $(target_elf) > hvisor.S
$(target_bin): elf
	$(OBJCOPY) $(target_elf) --strip-all -O binary $@
run: all
	cd qemu-test/host && ./test.sh

monitor:
	gdb-multiarch vmlinux \
	-ex 'target remote:1234' \
	-ex 'continue'

#	-ex 'file $(target_elf)' \
#	-ex 'add-symbol-file $(guest_obj)' \