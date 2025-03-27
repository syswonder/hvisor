KDIR ?= 
ARCH ?= arm64
LOG ?= LOG_INFO
DEBUG ?= n
VIRTIO_GPU ?= n

export KDIR
export ARCH
export LOG
export VIRTIO_GPU

.PHONY: all help env tools driver clean transfer tftp transfer_nxp check-kdir

OUTPUT_DIR ?= output
TFTP_DIR ?= ~/tftp

all: tools driver

help:
	@echo "Compilation targets:"
	@echo "  all          - Build tools and drivers (default)"
	@echo "  tools        - Build command-line tools"
	@echo "  driver       - Build kernel modules"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Environment variables:"
	@echo "  ARCH=arm64|riscv  Target architecture (required)"
	@echo "  LOG=LEVEL        Log level (default: LOG_INFO)"
	@echo "  KDIR=path        Linux kernel source path (required)"
	@echo "  VIRTIO_GPU=y|n   Enable GPU support (default: n)"

env:
	git submodule update --init --recursive

check-kdir:
ifeq ($(KDIR),)
	$(error Linux kernel directory is not set. Please set environment variable 'KDIR')
endif

tools: env
	$(MAKE) -C tools all
	@mkdir -p $(OUTPUT_DIR)
	cp tools/hvisor $(OUTPUT_DIR)
	
driver: env check-kdir
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
	else \
		echo "No files found in $(OUTPUT_DIR), skipping copy."; \
	fi

transfer_nxp: all
	sudo cp ./tools/hvisor ~/tftp
	sudo cp ./tools/ivc_demo ~/tftp
	sudo cp ./tools/rpmsg_demo ~/tftp
	sudo cp ./driver/hvisor.ko ~/tftp
	sudo cp ./driver/ivc.ko ~/tftp

clean: check-kdir
	make -C tools clean
	make -C driver clean
	rm -rf $(OUTPUT_DIR)
