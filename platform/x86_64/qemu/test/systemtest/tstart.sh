#!/usr/bin/expect -f

# Set environment variables to support UTF-8
set env(LANG) "en_US.UTF-8"
send_user "\r============Starting automated script execution============\r"
spawn make ARCH=x86_64 BOARD=qemu MODE=release run

# Set timeout (adjust as needed)
set timeout 240
# set password [lindex $argv 0]

puts "\n============Testing hvisor startup and virtio daemon============\n"

# Test starting zone1
expect {
    "root@zone0:/# " {
        send "nohup ./hvisor virtio start virtio_cfg.json &\r"
        send "\r"
    }
    timeout {
        exit 1
    }
}
expect {
    "root@zone0:/# " {
        sleep 10
        send "./hvisor zone start ./zone1_linux.json\r"
    }
    timeout {
        exit 1
    }
}

# Test screen access to zone1
expect {
    "root@zone0:/# " {
        sleep 5
        send "screen /dev/pts/1\r"
        sleep 10
        send "\r"
        sleep 10
        send "\r"
    }
    timeout {
        exit 1
    }
}
expect {
    "root@zone1:/# " {
        send "\x01\x01d"
        send "\r"
    }
    timeout {
        exit 1
    }
}
# Test printing zone list after starting zone1
expect {
    "root@zone0:/# " {
        send "./hvisor zone list > ./test/testresult/test_zone_list2.txt\r"
    }
    timeout {
        exit 1
    }
}

# temporarily disable the problematic subtests
# Shutting down zone1
expect {
    "root@zone0:/# " {
        send "./hvisor zone shutdown -id 1\r"
    }
    timeout {
        exit 1
    }
}

# Test printing zone list after removing zone1
expect {
    "root@zone0:/# " {
        send "./hvisor zone list > ./test/testresult/test_zone_list1.txt\r"
    }
    timeout {
        exit 1
    }
}

expect {
    "root@zone0:/# " {
        send "echo \"Test out finish!!\"\r"
    }
    timeout {
        exit 1
    }
}

after 5000  # Delay 5 seconds
# Compare test results and print finally
expect {
    "root@zone0:/# " {
        send "./test/tresult.sh\r"
    }
    timeout {
        exit 1
    }
}

expect {
    "Error: Test fail. Exiting script." {
        exit 1
    }
    "All tests passed. Script is exiting normally." {
        exit 0
    }
}

# exit
expect eof
