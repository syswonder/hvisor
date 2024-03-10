# Find all .dts files in the current directory
DTS_FILES := $(wildcard *.dts)

# Replace the .dts file extension with .dtb for all files found
DTB_FILES := $(DTS_FILES:.dts=.dtb)

# Default target
all: $(DTB_FILES)

# Pattern rule: how to generate a .dtb from a .dts
%.dtb: %.dts
	dtc -I dts -O dtb $< -o $@

# Clean target to remove generated files
clean:
	rm -f $(DTB_FILES)
