#!/usr/bin/expect -f

# aarch64/qemu-gicv3 performance benchmark driver
# Runs zone0 mem/irq/net benchmarks and zone1 startup timing.
# Data collection only (no benchmark pass/fail assertion); exits non-zero on infra/timeout failures.

set env(LANG) "en_US.UTF-8"
send_user "\r============ hvisor Performance Benchmark (aarch64/qemu-gicv3) ============\r"
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

# ── Stage 1: U-Boot → bootm ──
expect {
    -re "(1 bootflow, 1 valid).*=>" {
        send "bootm 0x40400000 - 0x40000000\r"
    }
    timeout {
        fail "U-Boot prompt timeout"
    }
}

# ── Stage 2: Zone0 shell ──
expect {
    -re {job control turned off.*#} {
        send "bash\r"
    }
    timeout {
        fail "zone0 shell timeout"
    }
}

expect {
    "root@(none):/# " {
        send "cd /home/arm64\r"
    }
    timeout { fail "zone0 cd /home/arm64 timeout" }
}

expect {
    "root@(none):/home/arm64# " {
        send "mkdir -p test/perfresult\r"
    }
    timeout { fail "mkdir test/perfresult timeout" }
}

expect {
    "root@(none):/home/arm64# " {
        send "mount -t proc proc /proc 2>/dev/null || true; mount -t sysfs sysfs /sys 2>/dev/null || true; mkdir -p /dev/shm; mount -t tmpfs tmpfs /dev/shm 2>/dev/null || true\r"
    }
    timeout { fail "prepare /proc /sys /dev/shm timeout" }
}

# ── Stage 3: Zone0 memory benchmark ──
send_user "\n\n============ Zone0: Memory Benchmark ============\n"
expect {
    "root@(none):/home/arm64# " {
        send "./test/bench/bench_mem.sh\r"
    }
    timeout { fail "bench_mem start timeout" }
}
set timeout 600
expect {
    "=== Done ===" { send_user "\n\[mem bench done\]\n" }
    "root@(none):/home/arm64# " { fail "bench_mem exited without done marker" }
    timeout {
        fail "bench_mem timeout"
    }
}
set timeout 600

# ── Stage 4: Zone0 IRQ / timer latency benchmark ──
send_user "\n\n============ Zone0: IRQ / Timer Latency Benchmark ============\n"
expect {
    "root@(none):/home/arm64# " {
        send "./test/bench/bench_irq.sh\r"
    }
    timeout { fail "bench_irq start timeout" }
}
set timeout 120
expect {
    "=== Done ===" { send_user "\n\[irq bench done\]\n" }
    timeout {
        fail "bench_irq timeout"
    }
}
set timeout 600

# ── Stage 5: Zone0 network benchmark ──
send_user "\n\n============ Zone0: Network Benchmark ============\n"
expect {
    "root@(none):/home/arm64# " {
        send "./test/bench/bench_net.sh\r"
    }
    timeout { fail "bench_net start timeout" }
}
set timeout 120
expect {
    "=== Done ===" { send_user "\n\[net bench done\]\n" }
    timeout {
        fail "bench_net timeout"
    }
}
set timeout 600

# ── Stage 6: Load hvisor kernel module ──
send_user "\n\n============ Loading hvisor.ko ============\n"
expect {
    "root@(none):/home/arm64# " {
        send "insmod hvisor.ko\r"
    }
    timeout { fail "insmod hvisor.ko timeout" }
}
expect {
    "root@(none):/home/arm64# " {
        # send "dmesg | tail -n 2 | awk -F ']' '{print \$2}' > ./test/testresult/test_insmod.txt\r"
        send "./test/textract_dmesg.sh ./test/testresult/test_insmod.txt\r"
    }
    timeout {
        fail "extract insmod dmesg timeout"
    }
}

# ── Stage 7: Start zone1 and measure startup time ──
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
    timeout {
        fail "zone1 boot timeout"
    }
}
set zone1_end [clock milliseconds]
set zone1_startup_ms [expr {$zone1_end - $zone1_start}]
send_user "\nzone1 startup time: ${zone1_startup_ms} ms\n"

# ── Stage 8: Shutting down zone1 ──
expect {
    "root@(none):/home/arm64# " {
        send "./hvisor zone shutdown -id 1\r"
    }
    timeout {
        fail "zone1 shutdown timeout"
    }
}

# ── Stage 9: Print summary ──
send_user "\n\n============ Benchmark Summary ============\n"
send_user "Zone1 startup time: ${zone1_startup_ms} ms\n"
expect {
    "root@(none):/home/arm64# " {
        send "echo '--- perf results ---' && cat test/perfresult/*.txt 2>/dev/null || echo '(no result files in zone0 perfresult)'\r"
        send_user "\n============ hvisor Performance Benchmark Finished ============\n"
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
    timeout { fail "print benchmark summary timeout" }
}
