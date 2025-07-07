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
// Authors:
//
//! SBI call wrappers

use super::cpu::ArchCpu;
use crate::arch::csr::*;
use crate::event::{send_event, IPI_EVENT_SEND_IPI, IPI_EVENT_WAKEUP};
use crate::hypercall::HyperCall;
use crate::percpu::{get_cpu_data, this_cpu_data};
use core::sync::atomic::{self, Ordering};
use riscv::register::sie;
use riscv_h::register::hvip;
use sbi_rt::{HartMask, SbiRet};
use sbi_spec::binary::{
    RET_ERR_ALREADY_AVAILABLE, RET_ERR_FAILED, RET_ERR_NOT_SUPPORTED, RET_SUCCESS,
};
use sbi_spec::{base, hsm, legacy, rfnc, spi, time};

// Reserved for hvisor-tool.
pub const EID_HVISOR: usize = 0x114514;

// Hvisor supported SBI extensions.
pub const NUM_EXT: usize = 6;
pub const EXT_TABLE: [usize; NUM_EXT] = [
    base::EID_BASE,
    time::EID_TIME,
    hsm::EID_HSM,
    spi::EID_SPI,
    rfnc::EID_RFNC,
    EID_HVISOR,
];

/// Use sbi call to putchar in console (qemu uart handler)
pub fn sbi_console_putchar(c: u8) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as _);
}

/// Use sbi call to getchar from console (qemu uart handler)
pub fn sbi_console_getchar() -> Option<u8> {
    #[allow(deprecated)]
    match sbi_rt::legacy::console_getchar() {
        x if x <= 0xff => Some(x as _),
        _ => None,
    }
}

/// Handle SBI calls form VS mode, sscratch -> current_cpu
pub fn sbi_vs_handler(current_cpu: &mut ArchCpu) {
    // a7 encodes the SBI extension ID (EID)
    // a6 encodes the SBI function ID (FID)
    let eid: usize = current_cpu.x[17];
    let fid: usize = current_cpu.x[16];
    let sbi_ret;
    match eid {
        // Base Extension (SBI Spec Chapter 4)
        base::EID_BASE => {
            sbi_ret = sbi_base_handler(fid, current_cpu);
        }
        // Timer Extension (SBI Spec Chapter 6)
        time::EID_TIME => {
            sbi_ret = sbi_time_handler(fid, current_cpu);
        }
        // Hart State Management (SBI Spec Chapter 9)
        hsm::EID_HSM => {
            sbi_ret = sbi_hsm_handler(fid, current_cpu);
        }
        // IPI Extension (SBI Spec Chapter 7)
        spi::EID_SPI => {
            sbi_ret = sbi_ipi_handler(fid, current_cpu);
        }
        // RFENCE Extension (SBI Spec Chapter 8)
        rfnc::EID_RFNC => {
            sbi_ret = sbi_rfence_handler(fid, current_cpu);
        }
        // Hvisor Extension (Hvisor Defined)
        EID_HVISOR => {
            sbi_ret = sbi_hvisor_handler(current_cpu);
        }
        // Legacy::Console putchar (usually used), temporily don't support other legacy extensions.
        legacy::LEGACY_CONSOLE_PUTCHAR => {
            sbi_ret = SbiRet {
                error: sbi_rt::legacy::console_putchar(current_cpu.x[10] as _),
                value: 0,
            };
        }
        _ => {
            // Pass through SBI call
            warn!("Unsupported SBI extension {:#x} function {:#x}", eid, fid);
            sbi_ret = SbiRet {
                error: RET_ERR_NOT_SUPPORTED,
                value: 0,
            };
        }
    }
    // Write the return value back to the current_cpu
    current_cpu.x[10] = sbi_ret.error as usize;
    current_cpu.x[11] = sbi_ret.value as usize;
}

/// Handle SBI Base Extension calls.
pub fn sbi_base_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: RET_SUCCESS,
        value: 0,
    };
    match fid {
        base::GET_SBI_SPEC_VERSION => {
            // The minor number of the SBI specification is encoded in the low 24 bits
            // with the major number encoded in the next 7 bits.
            // Bit 31 must be 0 and is reserved for future expansion.
            sbi_ret.value = 0x0200_0000; // spec version 2.0
        }
        base::GET_SBI_IMPL_VERSION => {
            // The implementation ID is a 32-bit value that identifies the SBI implementation.
            // The implementation ID is defined by the SBI implementer.
            sbi_ret.value = sbi_rt::get_sbi_impl_version();
        }
        base::GET_SBI_IMPL_ID => {
            sbi_ret.value = sbi_rt::get_sbi_impl_id();
        }
        base::PROBE_EXTENSION => {
            // This function needs one arg(extesion_id).
            let ext_id = current_cpu.x[10];
            // Returns 0 if the given SBI extension ID (EID) is not available or 1 if it is available
            // unless defined as any other non-zero value by the implementation.
            if EXT_TABLE.contains(&ext_id) {
                sbi_ret.value = ext_id;
            }
        }
        base::GET_MVENDORID | base::GET_MARCHID | base::GET_MIMPID => {
            // Return a value that is legal for the mvendorid CSR and 0 is always a legal value for this CSR.
            // Return a value that is legal for the marchid CSR and 0 is always a legal value for this CSR.
            // Return a value that is legal for the mimpid CSR and 0 is always a legal value for this CSR.
            sbi_ret.value = 0;
        }
        _ => {
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_base_extension fid {:#x}", fid);
        }
    }
    sbi_ret
}

/// SBI Set Timer handler
pub fn sbi_time_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    if fid != time::SET_TIMER {
        warn!("SBI: unsupported sbi_time_extension fid {:#x}", fid);
        return SbiRet {
            error: RET_ERR_NOT_SUPPORTED,
            value: 0,
        };
    }
    let stime_value = current_cpu.x[10];
    if current_cpu.sstc {
        // Set the vstimecmp, and don't need to inject timer interrupt.
        write_csr!(CSR_VSTIMECMP, stime_value);
    } else {
        // Hvisor should receive timer interrupt and inject it to guest.
        sbi_rt::set_timer(stime_value as _);
        unsafe {
            // clear guest timer interrupt pending
            hvip::clear_vstip();
            // enable timer interrupt
            sie::set_stimer();
        }
    }
    SbiRet {
        error: RET_SUCCESS,
        value: 0,
    }
}

#[allow(unused)]
#[derive(Debug, PartialEq)]
pub enum HSM_STATUS {
    STARTED,
    STOPPED,
    START_PENDING,
    STOP_PENDING,
    SUSPENDED,
    SUSPEND_PENDING,
    RESUMING_PENDING,
}

/// SBI Hart State Management handler.
pub fn sbi_hsm_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: RET_SUCCESS,
        value: 0,
    };
    match fid {
        hsm::HART_START => {
            // hsm start
            sbi_ret = sbi_hsm_start_handler(current_cpu);
        }
        hsm::HART_STOP => {
            // Todo: support hart stop.
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_hsm_hart_stop");
        }
        hsm::HART_GET_STATUS => {
            // Todo: support hart get status.
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_hsm_hart_get_status");
        }
        hsm::HART_SUSPEND => {
            // Todo: support hart suspend.
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_hsm_hart_suspend");
        }
        _ => {
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_hsm_extension fid {:#x}", fid);
        }
    }
    sbi_ret
}

/// SBI Hart State Management start handler.
pub fn sbi_hsm_start_handler(current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: RET_SUCCESS,
        value: 0,
    };
    let cpuid = current_cpu.x[10]; // In hvisor, it is physical cpu id.
    let start_addr = current_cpu.x[11];
    let opaque = current_cpu.x[12];
    if cpuid == current_cpu.cpuid {
        sbi_ret.error = RET_ERR_ALREADY_AVAILABLE;
    } else {
        info!(
            "SBI: try to wake up cpu {} run@ {:#x}, opaque@ {:#x}",
            cpuid, start_addr, opaque
        );
        let target_cpu = get_cpu_data(cpuid);
        let _lock = target_cpu.ctrl_lock.lock();
        // Todo: use hsm status, not just hvisor defined power_on.
        if target_cpu.arch_cpu.power_on {
            sbi_ret.error = RET_ERR_FAILED;
        } else {
            // Set the target cpu to start-pending.
            target_cpu.cpu_on_entry = start_addr;
            target_cpu.dtb_ipa = opaque;
            // Insert memory fence.
            atomic::fence(atomic::Ordering::SeqCst);
            send_event(cpuid, 0, IPI_EVENT_WAKEUP);
        }
        drop(_lock);
    }
    sbi_ret
}

/// SBI IPI handler.
pub fn sbi_ipi_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    if fid != spi::SEND_IPI {
        warn!("SBI: unsupported sbi_ipi_extension fid {:#x}", fid);
        return SbiRet {
            error: RET_ERR_NOT_SUPPORTED,
            value: 0,
        };
    }
    // unsigned long hart_mask is a scalar bit-vector containing hartids
    // unsigned long hart_mask_base is the starting hartid from which the bit-vector must be computed.
    let hart_mask = current_cpu.x[10];
    let hart_mask_base = current_cpu.x[11];
    let hart_mask_bits = HartMask::from_mask_base(hart_mask, hart_mask_base);
    for cpu_id in 0..64 {
        // hart_mask is 64 bits
        if hart_mask_bits.has_bit(cpu_id) {
            // the second parameter is ignored is riscv64.
            send_event(cpu_id, 0, IPI_EVENT_SEND_IPI);
        }
    }
    SbiRet {
        error: RET_SUCCESS,
        value: 0,
    }
}

/// SBI RFENCE handler.
pub fn sbi_rfence_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: RET_SUCCESS,
        value: 0,
    };
    let hart_mask = current_cpu.x[10];
    let hart_mask_base = current_cpu.x[11];
    let start_addr = current_cpu.x[12];
    let size = current_cpu.x[13];
    let asid = current_cpu.x[14];
    let hart_mask_bits = HartMask::from_mask_base(hart_mask, hart_mask_base);
    match fid {
        rfnc::REMOTE_FENCE_I => {
            // Instructs remote harts to execute FENCE.I instruction.
            sbi_ret = sbi_rt::remote_fence_i(hart_mask_bits);
        }
        rfnc::REMOTE_SFENCE_VMA => {
            // Instruct the remote harts to execute one or more HFENCE.VVMA instructions, covering the range of
            // guest virtual addresses between start_addr and start_addr + size for current VMID (in hgatp CSR) of
            // calling hart.
            sbi_ret = sbi_rt::remote_hfence_vvma(hart_mask_bits, start_addr, size);
        }
        rfnc::REMOTE_SFENCE_VMA_ASID => {
            // Instruct the remote harts to execute one or more HFENCE.VVMA instructions, covering the range of
            // guest virtual addresses between start_addr and start_addr + size for the given ASID and current
            // VMID (in hgatp CSR) of calling hart.
            sbi_ret = sbi_rt::remote_hfence_vvma_asid(hart_mask_bits, start_addr, size, asid);
        }
        _ => {
            sbi_ret.error = RET_ERR_NOT_SUPPORTED;
            warn!("SBI: unsupported sbi_rfence_extension fid {:#x}", fid);
        }
    }
    sbi_ret
}

/// SBI hvisor handler.
pub fn sbi_hvisor_handler(current_cpu: &mut ArchCpu) -> SbiRet {
    // Todo: according SBI spec, use a6 instead of a0 to tranfer code is more reasonable.
    let (code, arg0, arg1) = (current_cpu.x[10], current_cpu.x[11], current_cpu.x[12]);
    let cpu_data = this_cpu_data();
    debug!(
        "Hvisor ecall from CPU{},code: {:#x?}, arg0: {:#x?}, arg1: {:#x?}",
        cpu_data.id, code, arg0, arg1
    );
    // Handle hvisor related hypercall.
    let result = HyperCall::new(cpu_data).hypercall(code as _, arg0 as _, arg1 as _);
    match result {
        Ok(ret) => SbiRet {
            error: RET_SUCCESS,
            value: ret as _,
        },
        Err(e) => {
            error!("Hvisor ecall error: {:#x?}", e);
            SbiRet {
                error: RET_ERR_FAILED,
                value: e.code() as _,
            }
        }
    }
}
