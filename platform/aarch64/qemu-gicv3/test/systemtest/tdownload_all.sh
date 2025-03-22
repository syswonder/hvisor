#!/bin/bash

# Split archive + independent image download/merge/unzip script
# Usage: ./download_all.sh

# Configuration parameters
RELEASE_NAME="v2025.03.04"
BASE_URL="https://github.com/CHonghaohao/hvisor_env_img/releases/download/$RELEASE_NAME"

# Split archive configuration (must be in order)
ZIP_PARTS=(
  "rootfs1.zip.001"
  "rootfs1.zip.002"
  "rootfs1.zip.003"
)
ZIP_OUTPUT="rootfs1.zip"
UNZIP_DIR="platform/aarch64/qemu-gicv3/image/virtdisk"          # Extraction directory
``
# Independent image configuration
TARGET_DIR="platform/aarch64/qemu-gicv3/image/kernel"   # Target directory path
IMAGE_FILE="${TARGET_DIR}/Image"     # Full image file path
IMAGE_URL="$BASE_URL/Image"

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

  # Check if final files exist # TODO: check the files, not the directory - wheatfox
  if [ -d "$UNZIP_DIR" ] && [ -f "$IMAGE_FILE" ]; then
    echo -e "${GREEN}All files already exist:\n- Image file: $IMAGE_FILE\n- Extracted directory: $UNZIP_DIR${NC}"
    exit 0
  fi

  # Parallel download split files
  echo -e "${YELLOW}Starting split file downloads (parallel: $PARALLEL_DOWNLOADS)...${NC}"
  for part in "${ZIP_PARTS[@]}"; do
    local url="$BASE_URL/$part"
    local output="$part"

    if [ -f "$output" ]; then
      echo -e "${GREEN}Part already exists: $output${NC}"
      continue
    fi

    ((i=i%PARALLEL_DOWNLOADS)); ((i++==0)) && wait
    (
      if download_file "$url" "$output"; then
        echo -e "${GREEN}Download completed: $output${NC}"
      else
        exit 1
      fi
    ) &
  done
  wait

  # Verify split file integrity
  for part in "${ZIP_PARTS[@]}"; do
    if [ ! -f "$part" ]; then
      echo -e "${RED}Missing part: $part${NC}"
      exit 1
    fi
  done

  # Merge split files
  if [ ! -f "$ZIP_OUTPUT" ]; then
    echo -e "${YELLOW}Merging split files -> $ZIP_OUTPUT ...${NC}"
    cat "${ZIP_PARTS[@]}" > "$ZIP_OUTPUT" || {
      echo -e "${RED}Merge failed!${NC}"
      exit 1
    }
  else
    echo -e "${GREEN}Using existing merged file: $ZIP_OUTPUT${NC}"
  fi

  # Unzip files
  if [ ! -d "$UNZIP_DIR" ]; then
    echo -e "${YELLOW}Extracting to directory: $UNZIP_DIR ...${NC}"
    unzip -q "$ZIP_OUTPUT" -d "$UNZIP_DIR" || {
      echo -e "${RED}Extraction failed! Possible reasons:\n1. Password protected\n2. Corrupted file${NC}"
      exit 1
    }
  fi

  # Download independent image
  echo -e "${YELLOW}Downloading image file: $IMAGE_FILE ...${NC}"
  mkdir -p "$TARGET_DIR" || {
    echo -e "${RED}Failed to create directory: $TARGET_DIR${NC}"
    exit 1
  }

  if [ -f "$IMAGE_FILE" ]; then
    echo -e "${GREEN}Image already exists: $IMAGE_FILE${NC}"
  else
    download_file "$IMAGE_URL" "$IMAGE_FILE" || {
        echo -e "${RED}Download failed: $IMAGE_FILE${NC}"
        exit 1
    }
  fi

  # Final verification
  echo -e "\n${GREEN}All components ready: "
  echo -e "  - Image file: $(ls -lh $IMAGE_FILE)"
  echo -e "  - Extracted directory: $(du -sh $UNZIP_DIR)${NC}"
}

main
