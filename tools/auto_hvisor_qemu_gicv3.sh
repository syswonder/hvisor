#!/usr/bin/expect -f

# brief: Auto run hvisor on qemu-gicv3 platform and set basic network for root linux. 

spawn make ARCH=aarch64 LOG=info BOARD=qemu-gicv3 run

expect {
    -re "(1 bootflow, 1 valid).*=>" {
        send "bootm 0x40400000 - 0x40000000\r"
   }
   timeout {
        exit 1
   }
}


expect {
    -re {job control turned off.*#} {
        send "bash\r"
    }
    timeout {
        exit 1
    }
}


expect {
    "root@(none):/# " {
        send "cd /home/arm64\r\n\n"
    }
    timeout {
        exit 1
    }
}

# You can uncomment 'ntpdate cn.pool.ntp.org' to update the system time
expect {
    "root@(none):/home/arm64# " {
        send "mount -t proc proc /proc \r\n\nmount -t sysfs sysfs /sys \r\n\nip link set eth0 up \r\n\ndhclient eth0 \r\n\nbrctl addbr br0 \r\n\nbrctl addif br0 eth0 \r\n\nifconfig eth0 0 \r\n\ndhclient br0 \r\n\nip tuntap add dev tap0 mode tap \r\n\nbrctl addif br0 tap0 \r\n\nip link set dev tap0 up \r\n\n #ntpdate cn.pool.ntp.org \r\n\n"
    }
    timeout {
        exit 1
    }
}


interact

