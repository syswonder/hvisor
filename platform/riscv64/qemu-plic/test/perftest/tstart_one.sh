#!/usr/bin/expect -f

# riscv64/qemu-plic single benchmark driver
# Usage: PTEST=mem|irq|net|blk expect -f tstart_one.sh

set env(LANG) "en_US.UTF-8"
set run_exited_unexpectedly 1
set qemu_match "qemu-system-riscv64.*platform/riscv64/qemu-plic/image/virtdisk/rootfs1.ext4"

set ptest ""
if {[info exists env(PTEST)]} {
    set ptest $env(PTEST)
}
if {$ptest eq ""} {
    send_user "ERROR: PTEST is empty, use mem|irq|net|blk\n"
    exit 1
}

proc fail {msg} {
    global qemu_match
    send_user "\nERROR: $msg\n"
    catch {send "\x01x"}
    catch {close}
    catch {wait}
    catch {exec pkill -TERM -f -- $qemu_match}
    catch {exec pkill -KILL -f -- $qemu_match}
    exit 1
}

proc run_bench {name cmd timeout_sec result_file} {
    send_user "\n\n============ Zone0: $name Benchmark ============\n"
    expect {
        "root@(none):/home/riscv64# " {
            send "$cmd\r"
        }
        timeout { fail "$name start timeout" }
    }
    set timeout $timeout_sec
    expect {
        "=== Done ===" { send_user "\n\[$name bench done\]\n" }
        "root@(none):/home/riscv64# " { fail "$name exited without done marker" }
        timeout { fail "$name timeout" }
    }
    set timeout 600
    expect {
        "root@(none):/home/riscv64# " {
            send "echo '--- $name result ---' && cat $result_file 2>/dev/null || true\r"
        }
        timeout { fail "$name result print timeout" }
    }
}

proc run_blk_bench_in_zone1 {} {
    send_user "\n\n============ Zone1: Virtio-BLK Benchmark ============\n"

    expect {
        "root@(none):/home/riscv64# " {
            send "insmod hvisor.ko\r"
        }
        timeout { fail "insmod hvisor.ko timeout" }
    }

    expect {
        "root@(none):/home/riscv64# " {
            send "./boot_zone1.sh\r"
        }
        timeout { fail "boot_zone1 start timeout" }
    }
    expect {
        -re {Script started.*#} {
            send "bash\r"
        }
        timeout { fail "zone1 boot timeout" }
    }

    expect {
        "root@(none):/home/riscv64# " {
            send "./screen_zone1.sh\r"
            send "\r"
        }
        timeout { fail "screen_zone1 start timeout" }
    }
    expect {
        -re {\r?\n# } {
            send "bash\r"
        }
        timeout { fail "zone1 shell timeout" }
    }
    expect {
        "root@(none):/# " {
            send "cd /home/riscv64\r"
        }
        timeout { fail "zone1 cd /home/riscv64 timeout" }
    }
    expect {
        "root@(none):/home/riscv64# " {
            send "mkdir -p test/perfresult\r"
        }
        timeout { fail "zone1 mkdir test/perfresult timeout" }
    }
    expect {
        "root@(none):/home/riscv64# " {
            send "./test/bench/bench_blk.sh\r"
        }
        timeout { fail "bench_blk start timeout" }
    }
    set timeout 300
    expect {
        "=== Done ===" { send_user "\n\[blk bench done\]\n" }
        timeout { fail "bench_blk timeout" }
    }
    set timeout 600
    expect {
        "root@(none):/home/riscv64# " {
            send "echo '--- blk result (zone1) ---' && cat test/perfresult/bench_blk.txt 2>/dev/null || true\r"
        }
        timeout { fail "bench_blk result print timeout" }
    }
    expect {
        "root@(none):/home/riscv64# " {
            send "\x01\x01d"
        }
        timeout { fail "zone1 screen detach timeout" }
    }
    expect {
        "root@(none):/home/riscv64# " {
            send "./hvisor zone shutdown -id 1\r"
        }
        timeout { fail "zone1 shutdown timeout" }
    }
}

send_user "\r============ hvisor Single Benchmark (riscv64/qemu-plic, ptest=$ptest) ============\r"

catch {exec pkill -TERM -f -- $qemu_match}
after 300
spawn make run

set timeout 600
expect_before eof {
    if {$run_exited_unexpectedly} {
        fail "make run exited unexpectedly"
    }
}

expect {
    -re "char device redirected to /dev/pts.*(label X10007000)" {
        send "\x01c"
    }
    timeout { fail "pty redirect timeout" }
}

expect {
    "(qemu)" {
        send "c\r"
    }
    timeout { fail "QEMU monitor timeout" }
}

expect {
    -re {job control turned off.*#} {
        send "\x01cbash\r"
    }
    timeout { fail "zone0 shell timeout" }
}

expect {
    "root@(none):/# " {
        send "cd /home/riscv64\r"
    }
    timeout { fail "zone0 cd /home/riscv64 timeout" }
}

expect {
    "root@(none):/home/riscv64# " {
        send "mkdir -p test/perfresult\r"
    }
    timeout { fail "mkdir test/perfresult timeout" }
}

expect {
    "root@(none):/home/riscv64# " {
        send "mount -t proc proc /proc 2>/dev/null || true; mount -t sysfs sysfs /sys 2>/dev/null || true; mkdir -p /dev/shm; mount -t tmpfs tmpfs /dev/shm 2>/dev/null || true\r"
    }
    timeout { fail "prepare /proc /sys /dev/shm timeout" }
}

if {$ptest eq "mem"} {
    run_bench "Memory" "./test/bench/bench_mem.sh" 600 "test/perfresult/bench_mem.txt"
} elseif {$ptest eq "irq"} {
    run_bench "IRQ" "./test/bench/bench_irq.sh" 180 "test/perfresult/bench_irq.txt"
} elseif {$ptest eq "net"} {
    run_bench "Network" "./test/bench/bench_net.sh" 180 "test/perfresult/bench_net.txt"
} elseif {$ptest eq "blk"} {
    run_blk_bench_in_zone1
} else {
    fail "unsupported PTEST=$ptest, use mem|irq|net|blk"
}

send_user "\n============ Single Benchmark Finished ============\n"
set run_exited_unexpectedly 0
send "\x01x"
set timeout 20
expect {
    eof {}
    timeout {}
}
catch {close}
catch {wait}
exit 0
