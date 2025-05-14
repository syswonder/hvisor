



make BID=aarch64/ok62xx-c
dtc -I dts -o dtb ok6254.dts -o OK6254-C-0.dtb
dtc -I dts -o dtb zone1.dts -o OK6254-C-1.dtb



0
load mmc 1:1 0x80000000 OK6254-C-linux.dtb;load mmc 1:1 0x80400000 hvisor.bin;load mmc 1:1 0x82000000 Image;load mmc 1:1 0x88000000 OK6254-C-0.dtb


0
load mmc 1:1 0x80000000 OK6254-C-linux.dtb;load mmc 1:1 0x80400000 hvisor-gic.bin;load mmc 1:1 0x82000000 Image;load mmc 1:1 0x88000000 OK6254-C-0.dtb