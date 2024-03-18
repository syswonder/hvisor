#![allow(dead_code)]

//use crate::arch::riscv::csr::*;
use riscv::register::{hvip, sie};
pub mod SBI_EID {
    pub const SET_TIMER: usize = 0x54494D45;
    pub const EXTID_HSM: usize = 0x48534D;
    pub const SEND_IPI: usize = 0x735049;
}

use super::cpu::ArchCpu;

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
    let ret = sbi_call_5(
        current_cpu.x[17],
        current_cpu.x[16],
        current_cpu.x[10],
        current_cpu.x[11],
        current_cpu.x[12],
        current_cpu.x[13],
        current_cpu.x[14],
    );
    current_cpu.sepc += 4;
    current_cpu.x[10] = ret.0;
    current_cpu.x[11] = ret.1;
    trace!("sbi_call_5: error:{:#x}, value:{:#x}", ret.0, ret.1);
}
pub fn sbi_call_5(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> (usize, usize) {
    trace!("sbi_call_5: eid:{:#x}, fid:{:#x}", eid, fid);
    match eid {
        SBI_EID::SET_TIMER => {
            {
                info!("VS set timer");
                // write_csr!(CSR_HVIP, 0); //VSTIP
                // write_csr!(CSR_SIE, 1 << 9 | 1 << 5 | 1 << 1);
                set_timer(arg0);
                unsafe {
                    // clear guest timer interrupt pending
                    hvip::clear_vstip();
                    // enable timer interrupt
                    sie::set_stimer();
                }
                return (0, 0);
            }
        }
        //_ => sbi_ret = sbi_dummy_handler(),
        _ => debug!("Pass through SBI call eid {:#x} fid:{:#x}", eid, fid),
    }
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
    (error, value)
}

//other
pub const SBI_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_CONSOLE_GETCHAR: usize = 2;

pub const SBI_SET_TIMER: usize = 0;

pub const SBI_SUCCESS: usize = 0;
pub const SBI_ERR_FAILUER: isize = -1;
pub const SBI_ERR_NOT_SUPPORTED: isize = -2;
pub const SBI_ERR_INAVLID_PARAM: isize = -3;
pub const SBI_ERR_DENIED: isize = -4;
pub const SBI_ERR_INVALID_ADDRESS: isize = -5;
pub const SBI_ERR_ALREADY_AVAILABLE: isize = -6;

pub const SBI_EXTID_BASE: usize = 0x10;
pub const SBI_GET_SBI_SPEC_VERSION_FID: usize = 0;
pub const SBI_GET_SBI_IMPL_ID_FID: usize = 1;
pub const SBI_GET_SBI_IMPL_VERSION_FID: usize = 2;
pub const SBI_PROBE_EXTENSION_FID: usize = 3;
pub const SBI_GET_MVENDORID_FID: usize = 4;
pub const SBI_GET_MARCHID_FID: usize = 5;
pub const SBI_GET_MIMPID_FID: usize = 6;

pub const SBI_EXTID_TIME: usize = 0x54494D45;
pub const SBI_SET_TIMER_FID: usize = 0x0;

pub const SBI_EXTID_IPI: usize = 0x735049;
pub const SBI_SEND_IPI_FID: usize = 0x0;

pub const SBI_EXTID_HSM: usize = 0x48534D;
pub const SBI_HART_START_FID: usize = 0;
pub const SBI_HART_STOP_FID: usize = 1;
pub const SBI_HART_STATUS_FID: usize = 2;

pub const SBI_EXTID_RFNC: usize = 0x52464E43;
pub const SBI_REMOTE_FENCE_I_FID: usize = 0;
pub const SBI_REMOTE_SFENCE_VMA_FID: usize = 1;
pub const SBI_REMOTE_SFENCE_VMA_ASID_FID: usize = 2;
pub const SBI_REMOTE_HFENCE_GVMA_FID: usize = 3;
pub const SBI_REMOTE_HFENCE_GVMA_VMID_FID: usize = 4;
pub const SBI_REMOTE_HFENCE_VVMA_FIDL: usize = 5;
pub const SBI_REMOTE_HFENCE_VVMA_ASID_FID: usize = 6;
pub struct SbiRet {
    error: usize,
    value: usize,
}

#[inline(always)]
pub(crate) fn sbi_call_1(eid: usize, fid: usize, arg0: usize) -> SbiRet {
    let (error, value);
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            lateout("a1") value,
        );
    }
    SbiRet { error, value }
}

// pub fn sbi_vs_handler(current_cpu: &mut ArchCpu) {
//     let ext_id: usize = current_cpu.x[17];
//     let fid: usize = current_cpu.x[16];
//     let sbi_ret;

//     match ext_id {
//         SBI_EXTID_BASE => sbi_ret = sbi_base_handler(fid, current_cpu),
//         SBI_EXTID_TIME => sbi_ret = sbi_time_handler(current_cpu.x[10], fid),
//         SBI_CONSOLE_PUTCHAR => sbi_ret = sbi_console_putchar_handler(current_cpu.x[10]),
//         SBI_CONSOLE_GETCHAR => sbi_ret = sbi_console_getchar_handler(),
//         SBI_SET_TIMER => sbi_ret = sbi_legacy_set_time(current_cpu.x[10]),
//         //_ => sbi_ret = sbi_dummy_handler(),
//         _ => panic!("Unsupported SBI call id {:#x}", ext_id),
//     }
//     current_cpu.x[10] = sbi_ret.error;
//     current_cpu.x[11] = sbi_ret.value;
// }

// pub fn sbi_base_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
//     let mut sbi_ret = SbiRet {
//         error: SBI_SUCCESS,
//         value: 0,
//     };
//     match fid {
//         SBI_GET_SBI_SPEC_VERSION_FID => {
//             sbi_ret = sbi_call_1(SBI_EXTID_BASE, fid, 0);
//             debug!("GetSepcificationVersion: {}", sbi_ret.value);
//         }
//         SBI_GET_SBI_IMPL_ID_FID => {
//             sbi_ret.value = sbi_rt::get_sbi_impl_id();
//             debug!("GetImplementationId: {}", sbi_ret.value);
//         }
//         SBI_GET_SBI_IMPL_VERSION_FID => {
//             sbi_ret.value = sbi_rt::get_sbi_impl_version();
//             debug!("GetImplementationVersion: {}", sbi_ret.value);
//         }
//         SBI_PROBE_EXTENSION_FID => {
//             let extension = current_cpu.x[10];
//             sbi_ret = sbi_call_1(SBI_EXTID_BASE, fid, extension);
//             debug!("ProbeExtension: {}", sbi_ret.value);
//         }
//         SBI_GET_MVENDORID_FID => {
//             sbi_ret.value = sbi_rt::get_mvendorid();
//             debug!("GetVendorId: {}", sbi_ret.value);
//         }
//         SBI_GET_MARCHID_FID => {
//             sbi_ret.value = sbi_rt::get_marchid();
//             debug!("GetArchId: {}", sbi_ret.value);
//         }
//         SBI_GET_MIMPID_FID => {
//             sbi_ret.value = sbi_rt::get_mimpid();
//             debug!("GetMimpId: {}", sbi_ret.value);
//         }
//         _ => panic!("sbi base handler fid: {}", fid),
//     }
//     sbi_ret
// }

// pub fn sbi_console_putchar_handler(c: usize) -> SbiRet {
//     console_putchar(c);
//     return SbiRet {
//         error: SBI_SUCCESS,
//         value: 0,
//     };
// }

// pub fn sbi_console_getchar_handler() -> SbiRet {
//     let c = console_getchar();
//     return SbiRet {
//         error: SBI_SUCCESS,
//         value: c,
//     };
// }

// pub fn sbi_time_handler(stime: usize, fid: usize) -> SbiRet {
//     let mut sbi_ret = SbiRet {
//         error: SBI_SUCCESS,
//         value: 0,
//     };
//     if fid != SBI_SET_TIMER_FID {
//         sbi_ret.error = SBI_ERR_NOT_SUPPORTED as usize;
//         return sbi_ret;
//     }

//     // debug!("set timer: {}", stime);
//     set_timer(stime);
//     unsafe {
//         // clear guest timer interrupt pending
//         hvip::clear_vstip();
//         // enable timer interrupt
//         sie::set_stimer();
//     }
//     return sbi_ret;
// }

// pub fn sbi_rfence_handler(fid: usize) {

// }

// pub fn sbi_legacy_set_time(stime: usize) -> SbiRet {
//     let sbi_ret = SbiRet {
//         error: SBI_SUCCESS,
//         value: 0,
//     };
//     set_timer(stime);
//     unsafe {
//         // clear guest timer interrupt pending
//         hvip::clear_vstip();
//         // enable timer interrupt
//         sie::set_stimer();
//     }
//     return sbi_ret;
// }
