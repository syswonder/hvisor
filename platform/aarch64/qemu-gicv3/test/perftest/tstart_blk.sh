#!/usr/bin/expect -f

# aarch64/qemu-gicv3 standalone virtio-blk benchmark driver (zone1)
# Runs zone1 startup + screen enter + bench_blk + shutdown.

set env(LANG) "en_US.UTF-8"
send_user "\r============ hvisor Virtio-BLK Benchmark (aarch64/qemu-gicv3) ============\r"
set run_exited_unexpectedly 1

set qemu_match "qemu-system-aarch64.*platform/aarch64/qemu-gicv3/image/virtdisk/rootfs1.ext4"

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

# Clear stale QEMU from previous interrupted runs to avoid image lock.
catch {exec pkill -TERM -f -- $qemu_match}
after 300
spawn make run

set timeout 600
expect_before eof {
    if {$run_exited_unexpectedly} {
        fail "make run exited unexpectedly (likely QEMU start failure or image lock)"
    }
}

expect {
    -re "(1 bootflow, 1 valid).*=>" {
        send "bootm 0x40400000 - 0x40000000\r"
    }
    timeout { fail "U-Boot prompt timeout" }
}

expect {
    -re {job control turned off.*#} {
        send "bash\r"
    }
    timeout { fail "zone0 shell timeout" }
}

expect {
    "root@(none):/# " {
        send "cd /home/arm64\r"
    }
    timeout { fail "zone0 cd /home/arm64 timeout" }
}

expect {
    "root@(none):/home/arm64# " {
        send "insmod hvisor.ko\r"
    }
    timeout { fail "insmod hvisor.ko timeout" }
}

send_user "\n\n============ Zone1: Startup Time Measurement ============\n"
set zone1_start [clock milliseconds]
expect {
    "root@(none):/home/arm64# " {
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
set zone1_end [clock milliseconds]
set zone1_startup_ms [expr {$zone1_end - $zone1_start}]
send_user "\nzone1 startup time: ${zone1_startup_ms} ms\n"

send_user "\n\n============ Zone1: Virtio-BLK Benchmark ============\n"
expect {
    "root@(none):/home/arm64# " {
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
        send "cd /home/arm64\r"
    }
    timeout { fail "zone1 cd /home/arm64 timeout" }
}
expect {
    "root@(none):/home/arm64# " {
        send "mkdir -p test/perfresult\r"
    }
    timeout { fail "zone1 mkdir test/perfresult timeout" }
}
expect {
    "root@(none):/home/arm64# " {
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
    "root@(none):/home/arm64# " {
        send "echo '--- blk result (zone1) ---' && cat test/perfresult/bench_blk.txt 2>/dev/null || true\r"
    }
    timeout { fail "bench_blk result print timeout" }
}
expect {
    "root@(none):/home/arm64# " {
        send "\x01\x01d"
    }
    timeout { fail "zone1 screen detach timeout" }
}
expect {
    "root@(none):/home/arm64# " {
        send "./hvisor zone shutdown -id 1\r"
    }
    timeout { fail "zone1 shutdown timeout" }
}

send_user "\n\n============ Virtio-BLK Benchmark Summary ============\n"
send_user "Zone1 startup time: ${zone1_startup_ms} ms\n"
expect {
    "root@(none):/home/arm64# " {
        send_user "\n============ hvisor Virtio-BLK Benchmark Finished ============\n"
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
    }
    timeout { fail "final summary timeout" }
}
