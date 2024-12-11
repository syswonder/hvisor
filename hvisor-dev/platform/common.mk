DTS_FILES := $(wildcard $(PLAT_DIR)/dts/*.dts)

DTB_FILES := $(patsubst %.dts,%.dtb,$(DTS_FILES))

DTC := dtc

dtb: $(DTB_FILES)

%.dtb: %.dts
	$(DTC) -I dts -O dtb -o $@ $<
