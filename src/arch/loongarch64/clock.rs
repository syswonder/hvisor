use loongArch64::cpu::CPUCFG;
use loongArch64::time::*;
use spin::Mutex;

use crate::arch::cpu::this_cpu_id;

pub fn get_cpucfg_cc_freq() -> usize {
    let cpucfg = CPUCFG::read(0x4);
    cpucfg.get_bits(0, 31)
}

pub fn get_cpucfg_cc_mul() -> usize {
    let cpucfg = CPUCFG::read(0x5);
    cpucfg.get_bits(0, 15)
}

pub fn get_cpucfg_cc_div() -> usize {
    let cpucfg = CPUCFG::read(0x5);
    cpucfg.get_bits(16, 31)
}

pub fn read_stable_counter() -> usize {
    loongArch64::time::Time::read()
}

pub fn timer_test_tick() {
    if this_cpu_id() != 0 {
        return; // we only test on primary core
    }
    let freq = get_timer_freq();
    let start_time = read_stable_counter();
    info!(
        "loongarch64: clock: timer_test_tick: freq: {}, start_time: {}",
        freq, start_time
    );
    let mut last_log_time = start_time;
    while true {
        // after we passes 1 sec, we output a log, stop after 6 sec
        let current_time = read_stable_counter();
        if current_time - last_log_time > freq {
            info!(
                "loongarch64: clock: timer_test_tick: freq: {}, currentdebug_time: {}, calcuated seconds: {}",
                freq, current_time, (current_time - start_time) / freq
            );
            last_log_time = current_time;
        }
        if current_time - start_time > 10 * freq {
            break;
        }
    }
    info!(
        "loongarch64: clock: timer_test_tick: freq: {}, end_time: {}",
        freq,
        read_stable_counter()
    );
}
