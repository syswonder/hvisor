#!/bin/bash

# Configuration parameters
RELEASE_NAME="v2026.03.01"
BASE_URL="https://github.com/Solicey/hvisor_env_img_x86_64/releases/download/$RELEASE_NAME"

# Split archive configuration (must be in order)
ZIP_PARTS=(
  "rootfs1.zip"
)
ROOTFS_SRC_PATH="/home/sora/actions-runner/rootfs1.img"
ZIP_OUTPUT="rootfs1.zip"
UNZIP_DIR="platform/x86_64/qemu/image/virtdisk"          # Extraction directory
ROOTFS_FILE="${UNZIP_DIR}/rootfs1.img"

# Independent image configuration
TARGET_DIR="platform/x86_64/qemu/image/kernel"   # Target directory path
SETUP_FILE="${TARGET_DIR}/setup.bin"     # setup.bin file path
SETUP_URL="$BASE_URL/setup.bin"
VMLINUX_FILE="${TARGET_DIR}/vmlinux.bin"     # vmlinux.bin file path
VMLINUX_URL="$BASE_URL/vmlinux.bin"

# Download control parameters
MAX_RETRIES=3                # Max retries per file
PARALLEL_DOWNLOADS=1         # Parallel downloads (improves speed for large files)
TIMEOUT=3600                 # Timeout per file (seconds)

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Check dependencies
check_dependencies() {
  local missing=()
  command -v unzip >/dev/null 2>&1 || missing+=("unzip")
  command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1 || missing+=("curl/wget")

  if [ ${#missing[@]} -gt 0 ]; then
    echo -e "${RED}Error: Missing dependencies - ${missing[*]}${NC}"
    exit 1
  fi
}

# Download function with progress display
download_file() {
  local url="$1"
  local output="$2"
  local retries=0

  while [ $retries -lt $MAX_RETRIES ]; do
    if [ -f "$output" ]; then
      local current_size=$(stat -c%s "$output" 2>/dev/null || echo 0)
      if command -v curl >/dev/null 2>&1; then
        curl -C - -# -L --retry 2 --max-time $TIMEOUT -o "$output" "$url" && return 0
      elif command -v wget >/dev/null 2>&1; then
        wget -c -q --show-progress --tries=2 --timeout=$TIMEOUT -O "$output" "$url" && return 0
      fi
    else
      if command -v curl >/dev/null 2>&1; then
        curl -# -L --retry 2 --max-time $TIMEOUT -o "$output" "$url" && return 0
      elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress --tries=2 --timeout=$TIMEOUT -O "$output" "$url" && return 0
      fi
    fi

    ((retries++))
    echo -e "${YELLOW}Retry ($retries/$MAX_RETRIES): $output${NC}"
    sleep 2
  done

  echo -e "${RED}Download failed: $url${NC}"
  return 1
}

# Main process
main() {
  check_dependencies

  # Check if final files exist
  if [ -f "$ROOTFS_FILE" ] && [ -f "$SETUP_FILE" ] && [ -f "$VMLINUX_FILE" ]; then
    echo -e "${GREEN}All files already exist:\n- setup.bin: $SETUP_FILE\n- vmlinux.bin: $VMLINUX_FILE\n- rootfs file: $ROOTFS_FILE${NC}"
    exit 0
  fi

  # rootfs1.img
  mkdir -p "${UNZIP_DIR}"
  sudo cp "${ROOTFS_SRC_PATH}" "${ROOTFS_FILE}"

  # Download setup.bin
  echo -e "${YELLOW}Downloading setup.bin: $SETUP_FILE ...${NC}"
  mkdir -p "$TARGET_DIR" || {
    echo -e "${RED}Failed to create directory: $TARGET_DIR${NC}"
    exit 1
  }

  if [ -f "$SETUP_FILE" ]; then
    echo -e "${GREEN}setup.bin already exists: $SETUP_FILE${NC}"
  else
    echo -e "${GREEN}SETUP_URL: $SETUP_URL${NC}"
    download_file "$SETUP_URL" "$SETUP_FILE" || {
        echo -e "${RED}Download failed: $SETUP_FILE${NC}"
        exit 1
    }
  fi

  # Download vmlinux.bin
  echo -e "${YELLOW}Downloading vmlinux.bin: $VMLINUX_FILE ...${NC}"
  mkdir -p "$TARGET_DIR" || {
    echo -e "${RED}Failed to create directory: $TARGET_DIR${NC}"
    exit 1
  }

  if [ -f "$VMLINUX_FILE" ]; then
    echo -e "${GREEN}vmlinux.bin already exists: $VMLINUX_FILE${NC}"
  else
    echo -e "${GREEN}VMLINUX_URL: $VMLINUX_URL${NC}"
    download_file "$VMLINUX_URL" "$VMLINUX_FILE" || {
        echo -e "${RED}Download failed: $VMLINUX_FILE${NC}"
        exit 1
    }
  fi

  # Final verification
  echo -e "\n${GREEN}All components ready: "
  echo -e "  - setup.bin file: $(ls -lh $SETUP_FILE)"
  echo -e "  - vmlinux.bin file: $(ls -lh $VMLINUX_FILE)"
  echo -e "  - Extracted directory: $(du -sh $UNZIP_DIR)${NC}"
}

main