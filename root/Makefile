KDIR ?=
ARCH ?= arm64
LOG ?= LOG_INFO
DEBUG ?= n
VIRTIO_GPU ?= n
LIBC ?= gnu

export KDIR
export ARCH
export LOG
export VIRTIO_GPU
export LIBC

.PHONY: all help env tools driver clean transfer tftp transfer_nxp check-kdir check-config-change

OUTPUT_DIR ?= output
TFTP_DIR ?= ~/tftp

BUILD_VARS_STRING := arch=$(ARCH)-log=$(LOG)-debug=$(DEBUG)-virtio_gpu=$(VIRTIO_GPU)-libc=$(LIBC)-kdir=$(KDIR)
BUILD_CONFIG_FILE := $(OUTPUT_DIR)/.build_config

all: check-config-change tools driver
	@echo "Build successful. Storing current configuration."
	@echo "$(BUILD_VARS_STRING)" > $(BUILD_CONFIG_FILE)

help:
	@echo "Compilation targets:"
	@echo "  all          - Build tools and drivers (default)"
	@echo "  tools        - Build command-line tools"
	@echo "  driver       - Build kernel modules"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Environment variables:"
	@echo "  ARCH=arm64|riscv|loongarch   Target architecture (required)"
	@echo "  LOG=LEVEL                    Log level (default: LOG_INFO)"
	@echo "  KDIR=path                    Linux kernel source path (required)"
	@echo "  VIRTIO_GPU=y|n               Enable GPU support (default: n)"

env:
	git submodule update --init --recursive

check-kdir:
ifeq ($(KDIR),)
	$(error Linux kernel directory is not set. Please set environment variable 'KDIR')
endif

check-config-change:
	@mkdir -p $(OUTPUT_DIR)
	@if [ -f "$(BUILD_CONFIG_FILE)" ] && [ "$$(cat $(BUILD_CONFIG_FILE))" != "$(BUILD_VARS_STRING)" ]; then \
		echo "Build options changed. Forcing 'clean'."; \
		$(MAKE) clean; \
	fi

tools: env check-config-change
	$(MAKE) -C tools all
	@mkdir -p $(OUTPUT_DIR)
	cp tools/hvisor $(OUTPUT_DIR)

driver: env check-kdir check-config-change
	$(MAKE) -C driver all
	@mkdir -p $(OUTPUT_DIR)
	cp driver/hvisor.ko $(OUTPUT_DIR)

transfer: all
	./trans_file.sh ./tools/hvisor
	./trans_file.sh ./driver/hvisor.ko

tftp:
	@mkdir -p $(OUTPUT_DIR)
	@if [ -n "$(wildcard $(OUTPUT_DIR)/*)" ]; then \
		cp $(OUTPUT_DIR)/* $(TFTP_DIR); \
		echo "Copied files to $(TFTP_DIR):"; \
		ls -1 $(OUTPUT_DIR); \
	else \
		echo "No files found in $(OUTPUT_DIR), skipping copy."; \
	fi

clean: check-kdir
	@echo "Cleaning sub-projects and output directory..."
	@$(MAKE) -C tools clean
	@$(MAKE) -C driver clean
	@rm -rf $(OUTPUT_DIR)

fmt:
# if clang-format is not installed, ask to install
ifeq ($(shell which clang-format),)
	$(error clang-format is not installed. Please install it.)
endif
	find ./tools/ ./include/ ./driver/ -name "*.c" ! -name "*.mod.c" -o -name "*.h" | xargs clang-format -i
