ARCH ?= x86_64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug
BOARD ?= nuc14mnk
FEATURES=
BID ?=

# if user uses `make ID=aarch64/qemu-gicv2`, we parse it into ARCH and BOARD
ifeq ($(BID),)
	ARCH := $(ARCH)
	BOARD := $(BOARD)
else
	ARCH := $(shell echo $(BID) | cut -d'/' -f1)
	BOARD := $(shell echo $(BID) | cut -d'/' -f2)
endif

# if user add FEATURES in environment, we use it
# else we use the default FEATURES in platform/$(ARCH)/$(BOARD)/cargo/features
ifeq ($(FEATURES),)
    FEATURES := $(shell ./tools/read_features.sh $(ARCH) $(BOARD) || echo ERROR)
    ifeq ($(FEATURES),ERROR)
        $(error ERROR: Read FEATURES failed. Please check if ARCH="$(ARCH)" and BOARD="$(BOARD)" are correct.)
    endif
endif

ifeq ($(ARCH),aarch64)
    RUSTC_TARGET := aarch64-unknown-none
	GDB_ARCH := aarch64
else ifeq ($(ARCH),riscv64)
	RUSTC_TARGET := riscv64gc-unknown-none-elf
	GDB_ARCH := riscv:rv64
else ifeq ($(ARCH),loongarch64)
	RUSTC_TARGET := loongarch64-unknown-none
	GDB_ARCH := loongarch64
else ifeq ($(ARCH),x86_64)
	RUSTC_TARGET := x86_64-unknown-none
	GDB_ARCH := i386:x86-64
else
	$(error ERROR: Unsupported ARCH value: $(ARCH))
endif

OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

export MODE
export LOG
export ARCH
export BOARD
export BID
export RUSTC_TARGET
export FEATURES

# Build paths
build_path := target/$(RUSTC_TARGET)/$(MODE)
hvisor_elf := $(build_path)/hvisor
hvisor_bin := $(build_path)/hvisor.bin
image_dir  := platform/$(ARCH)/$(BOARD)/image

# Build arguments
build_args := 
build_args += --features "$(FEATURES)" 
build_args += --target $(RUSTC_TARGET)
build_args += -Z build-std=core,alloc
build_args += -Z build-std-features=compiler-builtins-mem

ifeq ($(MODE), release)
  build_args += --release
endif

# color code
COLOR_GREEN := $(shell tput setaf 2)
COLOR_RED := $(shell tput setaf 1)
COLOR_YELLOW := $(shell tput setaf 3)
COLOR_BLUE := $(shell tput setaf 4)
COLOR_BOLD := $(shell tput bold)
COLOR_RESET := $(shell tput sgr0)

# Targets
.PHONY: all elf disa run gdb monitor clean tools rootfs
all: clean_check gen_cargo_config $(hvisor_bin)
	@printf "\n"
	@printf "$(COLOR_GREEN)$(COLOR_BOLD)hvisor build summary:$(COLOR_RESET)\n"
	@printf "%-10s %s\n" "ARCH            =" "$(COLOR_BOLD)$(ARCH)$(COLOR_RESET)"
	@printf "%-10s %s\n" "BOARD           =" "$(COLOR_BOLD)$(BOARD)$(COLOR_RESET)"
	@printf "%-10s %s\n" "BID             =" "$(COLOR_BOLD)$(BID)$(COLOR_RESET)"
	@printf "%-10s %s\n" "LOG             =" "$(COLOR_BOLD)$(LOG)$(COLOR_RESET)"
	@printf "%-10s %s\n" "FEATURES        =" "$(COLOR_BOLD)$(FEATURES)$(COLOR_RESET)"
	@printf "%-10s %s\n" "RUSTC_TARGET    =" "$(COLOR_BOLD)$(RUSTC_TARGET)$(COLOR_RESET)"
	@printf "%-10s %s\n" "BUILD_PATH      =" "$(COLOR_BOLD)$(build_path)$(COLOR_RESET)"
	@printf "%-10s %s\n" "HVISON_BIN_SIZE =" "$(COLOR_BOLD)$(shell du -h $(hvisor_bin) | cut -f1)$(COLOR_RESET)"
	@printf "%-10s %s\n" "BUILD TIME      =" "$(COLOR_BOLD)$(shell date)$(COLOR_RESET)"
	@printf "\n"
	@printf "$(COLOR_GREEN)$(COLOR_BOLD)hvisor build success!$(COLOR_RESET)\n"

clean_check:
# if .config not exist, then everything is fine
# else we read .config and parse ARCH and BOARD, if they are different, we clean the build
	@if [ -f ".config" ]; then \
		CONFIG_ARCH=$$(cat .config | grep "ARCH" | cut -d'=' -f2); \
		CONFIG_BOARD=$$(cat .config | grep "BOARD" | cut -d'=' -f2); \
		if [ "$$CONFIG_ARCH" != "$(ARCH)" ] || [ "$$CONFIG_BOARD" != "$(BOARD)" ]; then \
			echo "$(COLOR_YELLOW)$(COLOR_BOLD)ARCH or BOARD changed(OLD: $$CONFIG_ARCH/$$CONFIG_BOARD, NEW: $(ARCH)/$(BOARD)), cleaning...$(COLOR_RESET)"; \
			./tools/clean.sh; \
		fi; \
	fi

gen_cargo_config:
	@printf "$(COLOR_GREEN)$(COLOR_BOLD)generating .cargo/config.toml...$(COLOR_RESET)\n"
	./tools/gen_cargo_config.sh
	@printf "$(COLOR_GREEN)$(COLOR_BOLD)generating .cargo/config.toml success!$(COLOR_RESET)\n"

elf:
	cargo build $(build_args)

disa:
	readelf -a $(hvisor_elf) > hvisor-elf.txt
	rust-objdump --disassemble --source --line-numbers $(hvisor_elf) > hvisor.asm

run: all
	$(QEMU) $(QEMU_ARGS)

gdb: all
	$(QEMU) $(QEMU_ARGS) -s -S

show-features:
	rustc --print=target-features --target=$(RUSTC_TARGET)

monitor:
	gdb-multiarch \
		-ex 'file $(hvisor_elf)' \
		-ex 'set arch $(GDB_ARCH)' \
		-ex 'target remote:1234'

jlink-server:
	JLinkGDBServer -select USB -if JTAG -device Cortex-A53 -port 1234

cp:
	cp $(hvisor_bin) ~/tftp

test-pre: download-test-img
	chmod +x platform/$(ARCH)/$(BOARD)/test/runner.sh
	@echo "added execute permission to test runner.sh for board $(BOARD)"

fmt-test: all
	cargo fmt --all -- --check
	@echo "cargo fmt check passed!"

fmt: all
	cargo fmt --all
	@echo "your code has been formatted"

clippy:
	cargo clippy $(build_args)

flash-img:
# run this will erase all environment for uboot, be careful
# the flash.img in repo will contains the correct bootcmd
	qemu-img create -f raw flash.img 64M

download-test-img:
# first check whether the file exists
	@if [ ! -f "flash.img" ]; then echo "\nflash.img not found, downloading...\n" && \
		wget https://github.com/enkerewpo/hvisor-uboot-env-img/releases/download/v20241227/flash.img.partial && \
		./tools/extract.sh ; \
	else echo "\nflash.img found\n"; \
	fi

test: clean test-pre gen_cargo_config
	cargo test $(build_args) -vv

stest: clean test-pre gen_cargo_config
	./platform/$(ARCH)/$(BOARD)/test/systemtest/tcompiledtb.sh
	./platform/$(ARCH)/$(BOARD)/test/systemtest/tdownload_all.sh
	./platform/$(ARCH)/$(BOARD)/test/systemtest/trootfs_deploy.sh
	./platform/$(ARCH)/$(BOARD)/test/systemtest/tstart.sh

dtb:
	@echo "building device tree at platform/$(ARCH)/$(BOARD)/image/dts"
	@if [ ! -d "platform/$(ARCH)/$(BOARD)/image/dts" ]; then echo "ERROR: dts directory not found"; exit 1; fi
	make -C platform/$(ARCH)/$(BOARD)/image/dts

clean:
	./tools/clean.sh

include platform/$(ARCH)/$(BOARD)/platform.mk