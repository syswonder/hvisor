# Basic settings
PLATFORM ?= qemu
ARCH ?= aarch64
LOG ?= info
STATS ?= off
PORT ?= 2333
MODE ?= debug

OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

# Check the value of ARCH and set corresponding RUSTC_TARGET and GDB_ARCH values.
ifeq ($(ARCH),aarch64)
	RUSTC_TARGET := aarch64-unknown-none
	GDB_ARCH := aarch64
else ifeq ($(ARCH),riscv64)
	RUSTC_TARGET := riscv64gc-unknown-none-elf
	GDB_ARCH := riscv:rv64
else ifeq ($(ARCH),loongarch64)
	RUSTC_TARGET := loongarch64-unknown-none
	GDB_ARCH := loongarch64
else
	# Error out if an unsupported ARCH value is provided.
	$(error Unsupported ARCH value: $(ARCH))
endif

# Export these variables so that they can be accessed in other parts of the build process (e.g., included scripts).
export MODE
export LOG
export ARCH
export PLATFORM

# Build paths
build_path := target/$(RUSTC_TARGET)/$(MODE)
hvisor_elf := $(build_path)/hvisor
hvisor_bin := $(shell pwd)/hvisor.bin

ifeq ($(ARCH),aarch64)
	hvisor_bin_command := if ! command -v mkimage > /dev/null; then \
		sudo apt update; sudo apt install u-boot-tools; \
		fi && \
		$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $(hvisor_bin).tmp && \
		mkimage -n hvisor_img -A arm64 -O linux -C none -a 0x40400000 -e 0x40400000 \
		-T kernel -d $(hvisor_bin).tmp $(hvisor_bin) && rm -rf $(hvisor_bin).tmp
else
	hvisor_bin_command := $(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
endif

# Build arguments
build_args := 
# Add the platform feature with the 'platform_' prefix concatenated with the PLATFORM value.
# This is to match the feature naming convention in the Cargo.toml file for building.
build_args += --features "platform_$(PLATFORM)" 
build_args += --target $(RUSTC_TARGET)
build_args += -Z build-std=core,alloc
build_args += -Z build-std-features=compiler-builtins-mem

ifeq ($(MODE), release)
  build_args += --release
endif

# Targets
# Declare these targets asphony to avoid conflicts with actual files (if any).
.PHONY: all elf disa run gdb monitor show-features jlink-server cp clean

all: $(hvisor_bin)

$(hvisor_bin): elf
	$(hvisor_bin_command)

elf:
	cargo build $(build_args)

disa:
	mkdir -p disa
	readelf -a $(hvisor_elf) > disa/hvisor-elf.txt
	rust-objdump --disassemble $(hvisor_elf) > disa/hvisor.S

show-features:
# Print the target features for the specified RUSTC_TARGET using rustc.
	rustc --print=target-features --target=$(RUSTC_TARGET)

monitor:
# Use gdb-multiarch to set up a debugging session for the hvisor ELF file.
# Set the architecture and connect to the remote target.
	gdb-multiarch \
		-ex 'file $(hvisor_elf)' \
		-ex 'set arch $(GDB_ARCH)' \
		-ex 'target remote:1234'

jlink-server:
# Start the JLinkGDBServer with specific options like selecting USB, JTAG interface, device, and port.
	JLinkGDBServer -select USB -if JTAG -device Cortex-A53 -port 1234

cp: all
# Copy the hvisor binary file to the specified location (~/tftp).
	cp $(hvisor_bin) ~/tftp

clean:
# Clean the build artifacts using cargo clean command.
	cargo clean

