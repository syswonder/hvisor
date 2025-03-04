#!/usr/bin/expect -f

# Set environment variables to support UTF-8
set env(LANG) "en_US.UTF-8"
send_user "\r============Starting automated script execution============\r"
spawn make run

# Set timeout (adjust as needed)
set timeout 240
# set password [lindex $argv 0]

# Wait for root password prompt and U-Boot prompt
expect {
#    "password for chh: " {
#         puts "\r============Handling sudo password and U-Boot commands============\r"
#         send "$password\r"
#         exp_continue
#    }
   -re "(1 bootflow, 1 valid).*=>" {
        # Enter command at prompt
        send "bootm 0x40400000 - 0x40000000\r"
   }
   timeout {
        exit 1
   }
}

puts "\n============Testing hvisor startup and virtio daemon============\n"

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
        send "cd /home/arm64\r"
    }
    timeout {
        exit 1
    }
}

# Test ls command
expect {
    "root@(none):/home/arm64# " {
        send "ls > ./test/testresult/test_ls.txt\r"
    }
    timeout {
        exit 1
    }
}

# Test pwd command
expect {
    "root@(none):/home/arm64# " {
        send "pwd > ./test/testresult/test_pwd.txt\r"
    }
    timeout {
        exit 1
    }
}

# Test kernel module loading
expect {
    "root@(none):/home/arm64# " {
        send "insmod hvisor.ko\r"
    }
    timeout {
        exit 1
    }
}
expect {
    "root@(none):/home/arm64# " {
        # send "dmesg | tail -n 2 | awk -F ']' '{print \$2}' > ./test/testresult/test_insmod.txt\r"
        send "./test/textract_dmesg.sh ./test/testresult/test_insmod.txt\r"
    }
    timeout {
        exit 1
    }
}


# Test starting zone1
expect {
    "root@(none):/home/arm64# " {
        send "./linux2.sh\r"
    }
    timeout {
        exit 1
    }
}
expect {
    -re {Script started, file is /dev/null.*#} {
        send "bash\r"
    }
    timeout {
        exit 1
    }
}
# Temporarily skip checking zone1 startup based on Log
expect {
    "root@(none):/home/arm64# " {
        send "dmesg | tail -n 3 | awk -F ']' '{print \$2}' > ./test/testresult/test_zone1_start.txt\r"
        send "./test/textract_dmesg.sh ./test/testresult/test_zone1_start.txt\r"
    }
    timeout {
        exit 1
    }
}

# Test screen access to zone1
expect {
    "root@(none):/home/arm64# " {
        send "./screen_linux2.sh\r"
        send "\r"
    }
    timeout {
        exit 1
    }
}
expect {
   "# " {
      send "bash\r"
   }
    timeout {
        exit 1
    }
}
# Variable to store zone1 ls command output
set test_zone1_ls ""
expect "root@(none):/# "
send "cd /home/arm64\r"
expect "root@(none):/home/arm64# "
# Send ls command and capture output to determine if zone1 is entered
send "ls | grep zone1.txt\r"
expect {
    -re {^[^\n]+\n(.*)\r\r\nroot@\(none\):/home/arm64# } {
        set test_zone1_ls $expect_out(1,string)
        send "\x01\x01d"
    }
    timeout {
        exit 1
    }
}
expect {
    "root@(none):/home/arm64# " {
        send "echo \"$test_zone1_ls\" > ./test/testresult/test_zone1_ls.txt\r"
    }
    timeout {
        exit 1
    }
}

# Test printing zone list after starting zone1
expect {
    "root@(none):/home/arm64# " {
        send "./hvisor zone list > ./test/testresult/test_zone_list2.txt\r"
    }
    timeout {
        exit 1
    }
}

# Shutting down zone1
expect {
    "root@(none):/home/arm64# " {
        send "./hvisor zone shutdown -id 1\r"
    }
    timeout {
        exit 1
    }
}

# Test printing zone list after removing zone1
expect {
    "root@(none):/home/arm64# " {
        send "./hvisor zone list > ./test/testresult/test_zone_list1.txt\r"
    }
    timeout {
        exit 1
    }
}

# expect {
#     "root@(none):/home/arm64# " {
#         send "echo \"Test out finish!!\"\r"
#     }
#     timeout {
#         exit 1
#     }
# }

after 5000  # Delay 5 seconds
# Compare test results and print finally
expect {
    "root@(none):/home/arm64# " {
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
exit 0
