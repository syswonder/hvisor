JH_DIR=~/jailhouse
JH=$JH_DIR/tools/jailhouse

sudo $JH disable
sudo rmmod jailhouse
sudo insmod $JH_DIR/driver/jailhouse.ko
sudo chown $(whoami) /dev/jailhouse
sudo $JH enable $JH_DIR/configs/x86/qemu-ubuntu.cell
