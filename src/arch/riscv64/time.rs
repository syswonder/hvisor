// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use crate::platform::__board::TIMEBASE_FREQ;
use riscv::register::time;

static mut TIMEBASE: u64 = 0;

/// Note: You should call this function once during initialization.
#[allow(unused)]
pub fn init_timebase() {
    unsafe {
        TIMEBASE = time::read() as u64;
    }
}

#[allow(unused)]
pub fn get_timebase() -> u64 {
    unsafe { TIMEBASE }
}

#[allow(unused)]
pub fn get_timefreq() -> u64 {
    TIMEBASE_FREQ
}

#[allow(unused)]
pub fn read_time() -> u64 {
    // Now only support 64-bit system.
    time::read() as u64
}

/// Return time in nanoseconds since some arbitrary point in the past.
#[allow(unused)]
pub fn get_time_ns() -> u64 {
    unsafe { (read_time() - TIMEBASE) * 1_000_000_000 / TIMEBASE_FREQ }
}

/// Return time in seconds since some arbitrary point in the past.
#[allow(unused)]
pub fn get_time_us() -> u64 {
    unsafe { (read_time() - TIMEBASE) * 1_000_000 / TIMEBASE_FREQ }
}

/// Return time in seconds since some arbitrary point in the past.
#[allow(unused)]
pub fn get_time_ms() -> u64 {
    unsafe { (read_time() - TIMEBASE) * 1_000 / TIMEBASE_FREQ }
}

/// Return time in seconds since some arbitrary point in the past.
#[allow(unused)]
pub fn get_time_s() -> u64 {
    unsafe { (read_time() - TIMEBASE) / TIMEBASE_FREQ }
}
