ARCH ?= aarch64
LOG ?=info
STATS ?= off
PORT ?= 2333

# default debug mode
MODE ?= debug

export MODE
export LOG
export ARCH
export STATS

OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

build_path := target/$(ARCH)/$(MODE)
target_elf := $(build_path)/rvmarm
target_bin := $(build_path)/rvmarm.bin

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
scp:
	scp -P $(PORT) -r $(target_bin) root@localhost:~/
.PHONY: disa
disa:
	rust-objdump --disassemble $(target_elf) > rvm.S
$(target_bin): elf
	$(OBJCOPY) $(target_elf) --strip-all -O binary $@
