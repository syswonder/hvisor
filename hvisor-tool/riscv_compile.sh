make all ARCH=riscv KDIR=/home/lgw/study/hypervisor/riscv/linux
./trans_file.sh ./tools/hvisor 
./trans_file.sh ./driver/hvisor.ko 