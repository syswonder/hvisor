DTS_FILES := $(wildcard *.dts)
DTB_FILES := $(DTS_FILES:.dts=.dtb)

all: $(DTB_FILES)

%.dts.1: %.dts
# preprocess all includes into a single dts with suffix .1
	cpp -I include -nostdinc -E -x assembler-with-cpp $< > $@

%.dtb: %.dts.1
	dtc -I dts -O dtb $< -o $@

clean:
	rm -f $(DTB_FILES)