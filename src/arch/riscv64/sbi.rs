//! SBI call wrappers

#![allow(unused)]
use crate::hypercall::{HyperCall};
use crate::percpu::{get_cpu_data, this_cpu_data};

use super::cpu::ArchCpu;
use crate::arch::csr::*;
use crate::event::{send_event, IPI_EVENT_WAKEUP};
use riscv::register::{hvip, sie};
#[allow(non_snake_case)]
pub mod SBI_EID {
    pub const BASE_EXTID: usize = 0x10;
    pub const SET_TIMER: usize = 0x54494D45;
    pub const EXTID_HSM: usize = 0x48534D;
    pub const SEND_IPI: usize = 0x735049;
    pub const RFENCE: usize = 0x52464E43;
    pub const PMU: usize = 0x504D55;
    pub const HVISOR: usize = 0x114514;
}
pub const SBI_SUCCESS: i64 = 0;
pub const SBI_ERR_FAILURE: i64 = -1;
pub const SBI_ERR_NOT_SUPPORTED: i64 = -2;
pub const SBI_ERR_INVALID_PARAM: i64 = -3;
pub const SBI_ERR_DENIED: i64 = -4;
pub const SBI_ERR_INVALID_ADDRESS: i64 = -5;
pub const SBI_ERR_ALREADY_AVAILABLE: i64 = -6;
pub struct SbiRet {
    error: i64,
    value: i64,
}
/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: u8) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as _);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> Option<u8> {
    #[allow(deprecated)]
    match sbi_rt::legacy::console_getchar() {
        x if x <= 0xff => Some(x as _),
        _ => None,
    }
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}

pub fn sbi_vs_handler(current_cpu: &mut ArchCpu) {
    let eid: usize = current_cpu.x[17];
    let fid: usize = current_cpu.x[16];
    let sbi_ret;
    match eid {
        //SBI_EXTID_BASE => sbi_ret = sbi_base_handler(fid, current_cpu),
        SBI_EID::BASE_EXTID => {
            trace!("SBI_EID::BASE,fid:{:#x}", fid);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::SET_TIMER => {
            //debug!("SBI_EID::SET_TIMER on CPU {}", current_cpu.cpuid);
            sbi_ret = sbi_time_handler(fid, current_cpu);
        }
        SBI_EID::EXTID_HSM => {
            info!("SBI_EID::EXTID_HSM on CPU {}", current_cpu.cpuid);
            sbi_ret = sbi_hsm_handler(fid, current_cpu);
        }
        SBI_EID::SEND_IPI => {
            trace!("SBI_EID::SEND_IPI on CPU {}", current_cpu.cpuid);
            trace!(
                "SBI_EID::SEND_IPI,cpuid:{:#x},mask:{:#x}",
                current_cpu.x[10],
                current_cpu.x[11]
            );
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::RFENCE => {
            trace!("SBI_EID::RFENCE,mask:{:#x}", current_cpu.x[10]);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::PMU => {
            trace!("SBI_EID::PMU,fid:{:#x}", fid);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::HVISOR => {
            trace!("SBI_EID::HVISOR,fid:{:#x}", fid);
            sbi_ret = sbi_hvisor_handler(current_cpu);
        }
        //_ => sbi_ret = sbi_dummy_handler(),
        _ => {
            warn!(
                "Pass through SBI call eid {:#x} fid:{:#x} on CPU {}",
                eid, fid, current_cpu.cpuid
            );
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
    }
    current_cpu.x[10] = sbi_ret.error as usize;
    current_cpu.x[11] = sbi_ret.value as usize;
}

pub fn sbi_call_5(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> SbiRet {
    let (error, value);
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
        );
    }
    SbiRet { error, value }
}

pub fn sbi_time_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    let stime = current_cpu.x[10];
    warn!("SBI_SET_TIMER stime: {:#x}", stime);
    if current_cpu.sstc {
        write_csr!(CSR_VSTIMECMP, stime);
    } else {
        set_timer(stime);
        unsafe {
            // clear guest timer interrupt pending
            hvip::clear_vstip();
            // enable timer interrupt
            sie::set_stimer();
        }
    }
    //debug!("SBI_SET_TIMER stime: {:#x}", stime);
    return sbi_ret;
}
pub fn sbi_hsm_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    match fid {
        0 => {
            // hsm start
            sbi_ret = sbi_hsm_start_handler(current_cpu);
        }
        _ => {
            error!("Unsupported HSM function {:#x}", fid);
        }
    }
    sbi_ret
}
pub fn sbi_hsm_start_handler(current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    let cpuid = current_cpu.x[10];

    if (cpuid == current_cpu.cpuid) {
        sbi_ret.error = SBI_ERR_ALREADY_AVAILABLE;
    } else {
        //TODO:add sbi conext in archcpu
        let cpuid = current_cpu.x[10];
        let start_addr = current_cpu.x[11];
        let opaque = current_cpu.x[12];

        info!("sbi: try to wake up cpu {} run@{:#x}", cpuid, start_addr);
        let target_cpu = get_cpu_data(cpuid);
        //todo add power_on check
        let _lock = target_cpu.ctrl_lock.lock();
        target_cpu.cpu_on_entry = start_addr;
        target_cpu.dtb_ipa = opaque;
        send_event(cpuid, 0, IPI_EVENT_WAKEUP);

        drop(_lock);
    }
    sbi_ret
}
pub fn sbi_hvisor_handler(current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    let (code, arg0, arg1) = (current_cpu.x[10], current_cpu.x[11], current_cpu.x[12]);

    let cpu_data = this_cpu_data();
    info!(
        "HVC from CPU{},code:{:#x?},arg0:{:#x?},arg1:{:#x?}",
        cpu_data.id, code, arg0, arg1
    );
    let result = HyperCall::new(cpu_data).hypercall(code as _, arg0 as _, arg1 as _);
    match result {
        Ok(ret) => {}
        Err(e) => {
            sbi_ret.error = SBI_ERR_FAILURE;
            sbi_ret.value = e.code() as _;
        }
    }
    sbi_ret
}
