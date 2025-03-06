# DEV_DIR, IMG_DIR, PLAT_DIR are defined in hvisor Makefile

UBOOT := $(IMG_DIR)/uboot.bin

FSIMG1 := $(IMG_DIR)/rootfs1.ext4
FSIMG2 := $(IMG_DIR)/rootfs2.ext4

zone0_kernel := $(IMG_DIR)/Image
zone0_dtb    := $(PLAT_DIR)/dts/zone0.dtb
zone1_kernel := $(IMG_DIR)/Image
zone1_dtb    := $(PLAT_DIR)/dts/zone1-linux.dtb

zone1_config := $(PLAT_DIR)/configs/zone1-linux.json

hvisor_bin   := $(IMG_DIR)/hvisor.bin

FS_FILE_LIST  := $(zone1_kernel) $(zone1_dtb) $(zone1_config)