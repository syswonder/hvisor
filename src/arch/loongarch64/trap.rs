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
//      Yulong Han <wheatfox17@icloud.com>
//      Ming Shen  <boneinscri@163.com>
//

use super::register::*;
use super::zone::ZoneContext;
use crate::arch::cpu::this_cpu_id;
use crate::arch::eiointc::{
    do_real_read_iocsr, do_real_write_iocsr, loongarch_eiointc_readl, loongarch_eiointc_writel,
    EIOINTC_BASE, EIOINTC_SIZE, EIOINTC_VIRT_BASE, EIOINTC_VIRT_SIZE,
};
use crate::arch::ipi::*;
use crate::arch::timer::{restore_timer, save_timer, timer_init};
use crate::consts::{IPI_EVENT_CLEAR_INJECT_IRQ, MAX_CPU_NUM};
use crate::cpu_data::{get_cpu_data, this_cpu_data};
use crate::device::irqchip::ls7a2000::chip::*;
use crate::device::irqchip::{inject_irq, ls7a2000::clear_irq};
use crate::device::virtio_trampoline::handle_virtio_irq;
use crate::event::{check_events, dump_cpu_events, dump_events};
use crate::hypercall::{SGI_IPI_ID, *};
use crate::memory::{addr, mmio_handle_access, MMIOAccess};
use crate::zone::Zone;

// IOCSR address range classification
const IOCSR_TYPE_IPI: usize = 0;
const IOCSR_TYPE_EIOINTC: usize = 1;
const IOCSR_TYPE_EIOINTC_VIRT: usize = 2;
const IOCSR_TYPE_OTHER: usize = 3;

fn get_iocsr_type(addr: usize) -> usize {
    if addr >= IOCSR_IPI_BASE && addr < IOCSR_IPI_BASE + 0x200 {
        IOCSR_TYPE_IPI
    } else if addr >= EIOINTC_BASE && addr < EIOINTC_BASE + EIOINTC_SIZE {
        IOCSR_TYPE_EIOINTC
    } else if addr >= EIOINTC_VIRT_BASE && addr < EIOINTC_VIRT_BASE + EIOINTC_VIRT_SIZE {
        IOCSR_TYPE_EIOINTC_VIRT
    } else {
        IOCSR_TYPE_OTHER
    }
}

// 0 or 7
// boneinscri : 2026.04
// VS_VALUE = 0, one handler
// VS_VALUE = 7, interrupt vector
// it can be changed runtime
pub const GLOBAL_VS_VALUE: usize = 0;

use crate::PHY_TO_DMW_UNCACHED;
use core::arch;
use core::arch::asm;
use core::panic;
use loongArch64::cpu;
use loongArch64::register;
use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::*;
use loongArch64::time;
use spin::Mutex;

pub struct TrapContextHelper {
    pub ecode: usize,
    pub esubcode: usize,
    pub is: usize,
    pub badv: usize,
    pub badi: usize,
    pub era: usize,
}

impl TrapContextHelper {
    pub const fn new() -> Self {
        Self {
            ecode: 0,
            esubcode: 0,
            is: 0,
            badv: 0,
            badi: 0,
            era: 0,
        }
    }

    pub fn update(
        &mut self,
        ecode: usize,
        esubcode: usize,
        is: usize,
        badv: usize,
        badi: usize,
        era: usize,
    ) {
        self.ecode = ecode;
        self.esubcode = esubcode;
        self.is = is;
        self.badv = badv;
        self.badi = badi;
        self.era = era;
    }
}

const GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU_INITDATA: Mutex<TrapContextHelper> =
    Mutex::new(TrapContextHelper::new());
pub static GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU: [Mutex<TrapContextHelper>; MAX_CPU_NUM] =
    [GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU_INITDATA; MAX_CPU_NUM];

pub fn install_trap_vector() {
    // force disable INT here
    // clear UEFI firmware's previous timer configs
    ticlr::clear_timer_interrupt();
    disable_global_interrupt();
    ecfg_ipi_disable();

    tcfg::set_en(false); // we may need to use timer irq to trap for our virtio clear injection
                         // only enable timer irq trap for debugging, because it may cause overheads for realtime nonroots

    // set CSR.EENTRY to _hyp_trap_vector and int vector offset to 0/?
    ecfg::set_vs(GLOBAL_VS_VALUE);
    eentry::set_eentry(_hyp_trap_vector as usize);

    // enable floating point
    euen::set_fpe(true); // basic floating point
    euen::set_sxe(true); // 128-bit SIMD
    euen::set_asxe(true); // 256-bit SIMD
}

/// enable CRMD.IE
#[inline(always)]
pub fn enable_global_interrupt() {
    crmd::set_ie(true);
}

/// disable CRMD.IE
#[inline(always)]
pub fn disable_global_interrupt() {
    crmd::set_ie(false);
}

#[inline(always)]
pub fn get_ms_counter(ms: usize) -> usize {
    ms * (time::get_timer_freq() / 1000)
}

#[inline(always)]
pub fn get_us_counter(us: usize) -> usize {
    us * (time::get_timer_freq() / 1000_000)
}

/// read the current stable counter value, not ns!
#[inline(always)]
pub fn ktime_get() -> usize {
    let mut current_counter_time;
    unsafe {
        asm!(
            "rdtime.d {}, {}",
            out(reg) current_counter_time,
            in(reg) 0,
        );
    }
    current_counter_time
}

pub fn ipi_init() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::IPI;
    ecfg::set_lie(lie_);
}

pub fn ecfg_timer_disable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ & !LineBasedInterrupt::TIMER;
    ecfg::set_lie(lie_);
}

pub fn ecfg_timer_enable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::TIMER;
    ecfg::set_lie(lie_);
}

pub fn ecfg_swi_enable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::SWI0 | LineBasedInterrupt::SWI1;
    ecfg::set_lie(lie_);
}

pub fn ecfg_swi_disable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ & !LineBasedInterrupt::SWI0 & !LineBasedInterrupt::SWI1;
    ecfg::set_lie(lie_);
}

pub fn ecfg_hwi_disable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ & !LineBasedInterrupt::HWI0;
    lie_ = lie_ & !LineBasedInterrupt::HWI1;
    lie_ = lie_ & !LineBasedInterrupt::HWI2;
    lie_ = lie_ & !LineBasedInterrupt::HWI3;
    lie_ = lie_ & !LineBasedInterrupt::HWI4;
    lie_ = lie_ & !LineBasedInterrupt::HWI5;
    lie_ = lie_ & !LineBasedInterrupt::HWI6;
    lie_ = lie_ & !LineBasedInterrupt::HWI7;
    ecfg::set_lie(lie_);
}

pub fn ecfg_hwi_enable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::HWI0;
    lie_ = lie_ | LineBasedInterrupt::HWI1;
    lie_ = lie_ | LineBasedInterrupt::HWI2;
    lie_ = lie_ | LineBasedInterrupt::HWI3;
    lie_ = lie_ | LineBasedInterrupt::HWI4;
    lie_ = lie_ | LineBasedInterrupt::HWI5;
    lie_ = lie_ | LineBasedInterrupt::HWI6;
    lie_ = lie_ | LineBasedInterrupt::HWI7;
    ecfg::set_lie(lie_);
}

/// Translate exception code to string
pub fn ecode2str(ecode: usize, esubcode: usize) -> &'static str {
    match ecode {
        0x0 => "INT(Interrupt)",
        0x1 => "PIL(Page Illegal Load)",
        0x2 => "PIS(Page Illegal Store)",
        0x3 => "PIF(Page Illegal Fetch)",
        0x4 => "PME(Page Modify Exception)",
        0x5 => "PNR(Page Not Readable)",
        0x6 => "PNX(Page Not Executable)",
        0x7 => "PPI(Page Privilege Illegal)",
        0x8 => match esubcode {
            0x0 => "ADEF(Instruction Fetch Address Exception)",
            0x1 => "ADEM(Memory Access Address Exception)",
            _ => "error_esubcode",
        },
        0x9 => "ALE(Address Misaligned Exception)",
        0xa => "BCE(Edge Check Exception)",
        0xb => "SYS(System Call Exception)",
        0xc => "BRK(Breakpoint Exception)",
        0xd => "INE(Instruction Not Exist)",
        0xe => "IPE(Instruction Privilege Exception)",
        0xf => "FPD(Floating Point Disabled)",
        0x10 => "SXD(128-bit SIMD Disabled)",
        0x11 => "ASXD(256-bit SIMD Disabled)",
        0x12 => match esubcode {
            0x0 => "FPE(Floating Point Exception)",
            0x1 => "VFPE(Vector Floating Point Exception)",
            _ => "error_esubcode",
        },
        0x13 => match esubcode {
            0x0 => "WPEF(Watchpoint Exception Fetch)",
            0x1 => "WPEM(Watchpoint Exception Memory)",
            _ => "error_esubcode",
        },
        0x14 => "BTD(Binary Translation Disabled)",
        0x15 => "BTE(Binary Translation Exception)",
        0x16 => "GSPR(Guest Sensitive Privileged Resource)",
        0x17 => "HVC(Hypervisor Call)",
        0x18 => match esubcode {
            0x0 => "GCSC(Guest CSR Software Change)",
            0x1 => "GCHC(Guest CSR Hardware Change)",
            _ => "error_esubcode",
        },
        _ => "reserved_ecode",
    }
}

fn handle_page_modify_fault() {
    let badv_ = badv::read();
    info!(
        "loongarch64: handling page modify exception, vaddr = 0x{:x}",
        badv_.vaddr()
    );
    info!("loongarch64: ignoring this exception, todo: set dirty bit in page table entry");
}

#[no_mangle]
pub fn trap_handler(mut ctx: &mut ZoneContext) {
    trace!("loongarch64: trap_handler: ctx addr = {:p}", &ctx);

    // save timer
    // --boneinscri 2026.04
    let pcpu_id = this_cpu_id();
    save_timer(ctx, pcpu_id);

    // dump trap csr regs
    let estat_ = estat::read();
    let ecode = estat_.ecode();
    let esubcode = estat_.esubcode();
    let is = estat_.is();
    let badv_ = badv::read();
    let badi_ = badi::read();
    let era_ = era::read();
    // TLB dump
    let tlbrera_ = tlbrera::read();
    let tlbrbadv_ = tlbrbadv::read();
    let tlbrelo0_ = tlbrelo0::read();
    let tlbrelo1_ = tlbrelo1::read();

    // update global trap context helper
    GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU[this_cpu_id()]
        .lock()
        .update(
            ecode,
            esubcode,
            is,
            badv_.vaddr(),
            badi_.inst() as usize,
            era_.raw(),
        );

    let mut is_idle = false;
    if ecode == ECODE_GSPR && badi_.inst() == 0b0000_0110_0100_1000_1000_0000_0000_0000 {
        is_idle = true;
        ctx.sepc += 4;
        // just return to guest
        unsafe {
            let _ctx_ptr = ctx as *mut ZoneContext;
            _vcpu_return(_ctx_ptr as usize);
        }
    }

    debug!(
            "loongarch64: trap_handler: {} ecode={:#x} esubcode={:#x} is={:#x} badv={:#x} badi={:#x} era={:#x}", 
            ecode2str(ecode, esubcode),
            ecode,
            esubcode,
            is,
            badv_.vaddr(),
            badi_.inst(),
            era_.raw(),
        );

    handle_exception(
        ecode,
        esubcode,
        era_.raw(),
        is,
        badi_.inst() as usize,
        badv_.vaddr(),
        ctx,
    );

    // restore timer + inject irq
    // --boneinscri 2026.04
    restore_timer(ctx, pcpu_id);
    deliver_irq();

    debug!("loongarch64: trap_handler: return");

    unsafe {
        let _ctx_ptr = ctx as *mut ZoneContext;
        _vcpu_return(_ctx_ptr as usize);
    }
}

const ECODE_INT: usize = 0x0;
const ECODE_GSPR: usize = 0x16;
const ECODE_PIL: usize = 0x1;
const ECODE_PIS: usize = 0x2;
const ECODE_HVC: usize = 0x17;
const ECODE_PNR: usize = 0x5;

fn handle_exception(
    ecode: usize,
    esubcode: usize,
    era: usize,
    is: usize,
    badi: usize,
    badv: usize,
    ctx: &mut ZoneContext,
) {
    match ecode {
        ECODE_INT => {
            debug!(
                "This is an interrupt exception, is={:#x}, ecfg.lie={:?}",
                is,
                ecfg::read().lie()
            );
            // INT = 0x0,   Interrupt
            handle_interrupt(is);
        }
        ECODE_GSPR => {
            // according to kvm's code, we should emulate the instruction that cause the GSPR exception - wheatfox 2024.4.12
            // GSPR = 0x16, Guest Sensitive Privileged Resource
            trace!(
                "This is a GSPR exception, badv={:#x}, badi={:#x}",
                badv,
                badi
            );
            // arch_send_event(1, 0x7);
            emulate_instruction(era, badi, ctx);
        }
        ECODE_HVC => {
            // HVC = 0x17,  Hypervisor Call
            // code = a0(r4), arg0 = a1(r5), arg1 = a2(r6)
            handle_hvc(ctx);
        }
        ECODE_PIL | ECODE_PIS | ECODE_PNR => {
            info!("exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                    ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv);
            // we first assume this lies in virtio region
            // since we didn't add these regions into VMM Pages
            /*
                LD.B    rd, rj, si12    0010100000  si12    rj5   rd5
                LD.H    rd, rj, si12    0010100001  si12    rj5   rd5
                LD.W    rd, rj, si12    0010100010  si12    rj5   rd5
                LD.D    rd, rj, si12    0010100011  si12    rj5   rd5
                ST.B    rd, rj, si12    0010100100  si12    rj5   rd5
                ST.H    rd, rj, si12    0010100101  si12    rj5   rd5
                ST.W    rd, rj, si12    0010100110  si12    rj5   rd5
                ST.D    rd, rj, si12    0010100111  si12    rj5   rd5
                LD.BU   rd, rj, si12    0010101000  si12    rj5   rd5
                LD.HU   rd, rj, si12    0010101001  si12    rj5   rd5
                LD.WU   rd, rj, si12    0010101010  si12    rj5   rd5
                LDPTR.W rd, rj, si14    00100100    si14    rj5   rd5
                STPTR.W rd, rj, si14    00100101    si14    rj5   rd5
                LDPTR.D rd, rj, si14    00100110    si14    rj5   rd5
                STPTR.D rd, rj, si14    00100111    si14    rj5   rd5
                LDX.B   rd, rj, rk      00111000000000 000 rk rj  rd5
                LDX.H   rd, rj, rk      00111000000001 000 rk rj  rd5
                LDX.W   rd, rj, rk      00111000000010 000 rk rj  rd5
                LDX.D   rd, rj, rk      00111000000011 000 rk rj  rd5
                STX.B   rd, rj, rk      00111000000100 000 rk rj  rd5
                STX.H   rd, rj, rk      00111000000101 000 rk rj  rd5
                STX.W   rd, rj, rk      00111000000110 000 rk rj  rd5
                STX.D   rd, rj, rk      00111000000111 000 rk rj  rd5
                LDX.BU  rd, rj, rk      00111000001000 000 rk rj  rd5
                LDX.HU  rd, rj, rk      00111000001001 000 rk rj  rd5
                LDX.WU  rd, rj, rk      00111000001010 000 rk rj  rd5
            */
            let ins = badi;
            let mut is_write = false;
            let mut is_u = false;
            let mut value = 0;
            let mut size = 0;
            let mut addr = 0;
            let mut target_rd_idx = 0;
            let prefix6 = extract_field(ins, 26, 6);
            if prefix6 == 0b001010 {
                // load/store
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let si12 = extract_field(ins, 10, 12);
                let ty = extract_field(ins, 24, 2); // ld/st/ldu - 0b00/0b01/0b10
                let sz = extract_field(ins, 22, 2); // 0b00=byte, 0b01=half, 0b10=word, 0b11=double
                match ty {
                    0b00 => {
                        // LD
                        is_write = false;
                    }
                    0b01 => {
                        // ST
                        is_write = true;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDU
                        is_write = false;
                        is_u = true;
                    }
                    _ => panic!("unhandled type"),
                }
                size = match sz {
                    0b00 => 1,
                    0b01 => 2,
                    0b10 => 4,
                    0b11 => 8,
                    _ => panic!("unhandled size"),
                };
            } else if prefix6 == 0b001001 {
                // load/store pointer
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let si14 = extract_field(ins, 10, 14);
                let mem_addr = ctx.x[rj] as usize + si14 as usize;
                let ty = extract_field(ins, 24, 2);
                match ty {
                    0b00 => {
                        // LDPTR.W
                        is_write = false;
                        size = 4;
                    }
                    0b01 => {
                        // STPTR.W
                        is_write = true;
                        size = 4;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDPTR.D
                        is_write = false;
                        size = 8;
                    }
                    0b11 => {
                        // STPTR.D
                        is_write = true;
                        size = 8;
                        value = ctx.x[rd];
                    }
                    _ => panic!("unhandled size"),
                }
            } else if prefix6 == 0b001110 {
                // load/store extended
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let rk = extract_field(ins, 10, 5);
                let sz = extract_field(ins, 18, 2);
                let ty = extract_field(ins, 20, 2);
                match ty {
                    0b00 => {
                        // LDX
                        is_write = false;
                    }
                    0b01 => {
                        // STX
                        is_write = true;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDXU
                        is_write = false;
                        is_u = true;
                    }
                    _ => panic!("unhandled type"),
                }
                size = match sz {
                    0b00 => 1,
                    0b01 => 2,
                    0b10 => 4,
                    0b11 => 8,
                    _ => panic!("unhandled size"),
                };
            } else {
                panic!("unhandled instruction: {:#b}/{:#x}", ins, ins);
            }

            let mut mmio_access = MMIOAccess {
                address: badv,
                size,
                is_write,
                value,
            };
            // debug!(
            //     "mmio_access, addr={:#x}, size={:#x}, is_write={}, value={:#x}",
            //     mmio_access.address, mmio_access.size, mmio_access.is_write, mmio_access.value
            // );
            debug!(
                "!!!! {} mmio_access@{:#x} s={:#x} v={:#x}",
                if is_write { "->write" } else { "<- read" },
                mmio_access.address,
                mmio_access.size,
                mmio_access.value
            );
            let res = mmio_handle_access(&mut mmio_access);
            match res {
                Ok(_) => {
                    debug!("handle mmio success, v={:#x}", mmio_access.value);
                    if !is_write {
                        // we read an usize from our zone0 virtio-daemon
                        // need to trim and extend it to 64-bit reg according to is_u and size
                        let mask = match mmio_access.size {
                            1 => 0xff,
                            2 => 0xffff,
                            4 => 0xffffffff,
                            8 => 0xffffffffffffffff,
                            _ => panic!("invalid mmio access size: {}", mmio_access.size),
                        };
                        let trimmed_by_size = mmio_access.value & mask;
                        let extended = if !is_u {
                            // normal instruction with no .u use sign extension
                            signed_ext(trimmed_by_size, mmio_access.size * 8)
                        } else {
                            // .u instruction zero extend
                            trimmed_by_size
                        };
                        debug!(
                            "read from mmio, raw={:#x}, trimmed={:#x}, extended={:#x}",
                            mmio_access.value, trimmed_by_size, extended
                        );
                        ctx.x[target_rd_idx] = extended;
                    }
                    // we should jump to next instruction because we 'emulated' the instruction
                    ctx.sepc += 4;
                }
                Err(e) => {
                    error!(
                        "mmio access failed, error = {:?}, this is a real page fault",
                        e
                    );
                    error!("unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                    ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv);
                    this_cpu_data().arch_cpu.idle(); // boneinscri 2026.04, use shutdown to restart it~ for debugging
                }
            }
        }
        _ => {
            error!("unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",  
            ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv);
            this_cpu_data().arch_cpu.idle(); // boneinscri 2026.04, use shutdown to restart it~ for debugging
        }
    }
}

fn signed_ext(value: usize, size: usize) -> usize {
    let sign_bit = 1 << (size - 1);
    if value & sign_bit != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

#[no_mangle]
pub fn _vcpu_return(ctx: usize) {
    let z = this_cpu_data().zone.as_ref();
    let vm_id;
    if z.is_none() {
        trace!("loongarch64: _vcpu_return: no zone found for cpu {}, maybe this is a kernel exception return", this_cpu_id());
        vm_id = 0;
    } else {
        // since LVZ use GID=0 for hypervisor TLB, we cannot use zone id 0 here
        // so we add it by 1 - wheatfox
        vm_id = z.unwrap().id() + 1;
    }
    gstat::set_gid(vm_id);
    gstat::set_pgm(true);
    trace!(
        "loongarch64: _vcpu_return: set hardware Guest ID to {} for zone {}",
        vm_id,
        z.unwrap().id()
    );
    // Configure guest TLB control
    gtlbc::set_use_tgid(true);
    gtlbc::set_tgid(vm_id);
    let gtlbc_ = gtlbc::read();
    trace!(
        "loongarch64: _vcpu_return: gtlbc.use_tgid = {}",
        gtlbc_.use_tgid()
    );
    trace!("loongarch64: _vcpu_return: gtlbc.tgid = {}", gtlbc_.tgid());
    // Configure guest control
    gcfg::set_matc(0x1);
    let gcfg_ = gcfg::read();
    // Disable GSPR guest sensitive privileged resource exception
    gcfg::set_topi(false);
    gcfg::set_toti(false);
    gcfg::set_toe(false);
    gcfg::set_top(false);
    gcfg::set_tohu(false);
    gcfg::set_toci(0x2);

    // when booting linux, linux is waiting for a HWI, but it never really comes
    // to guest vm, in JTAG it's already in host CSR: ESTATE=0000000000000004,which is HWI0(UART...)
    // so we need to relay host HWI to guest - wheatfox 2024.4.15

    gintc::set_hwip(0xff); // HWI7-HWI0 pass to guest

    // Enable interrupt
    prmd::set_pie(true);

    // ecfg_timer_enable();

    // ecfg_hwi_enable();
    // ecfg_swi_enable();

    ecfg::set_vs(GLOBAL_VS_VALUE);
    eentry::set_eentry(_hyp_trap_vector as usize);

    trace!(
        "loongarch64: _vcpu_return: calling _hyp_trap_return with ctx = {:#x}",
        ctx
    );
    unsafe {
        _hyp_trap_return(ctx);
    }
}

#[no_mangle]
#[naked]
#[link_section = ".trap_entry"]
extern "C" fn _hyp_trap_vector() {
    unsafe {
        asm!(
            "csrwr $r3, {LOONGARCH_CSR_DESAVE}",
            "csrrd $r3, {LOONGARCH_CSR_SAVE3}",
            //parpare VmContext for zone_trap_handler
            //save 32 GPRS except $r3
            //save gcsrs managed by guest
            "addi.d $r3, $r3, -768",
            "st.d $r0, $r3, 0",
            "st.d $r1, $r3, 8",
            "st.d $r2, $r3, 16",
            "st.d $r4, $r3, 32",
            "st.d $r5, $r3, 40",
            "st.d $r6, $r3, 48",
            "st.d $r7, $r3, 56",
            "st.d $r8, $r3, 64",
            "st.d $r9, $r3, 72",
            "st.d $r10, $r3, 80",
            "st.d $r11, $r3, 88",
            "st.d $r12, $r3, 96",
            "st.d $r13, $r3, 104",
            "st.d $r14, $r3, 112",
            "st.d $r15, $r3, 120",
            "st.d $r16, $r3, 128",
            "st.d $r17, $r3, 136",
            "st.d $r18, $r3, 144",
            "st.d $r19, $r3, 152",
            "st.d $r20, $r3, 160",
            "st.d $r21, $r3, 168",
            "st.d $r22, $r3, 176",
            "st.d $r23, $r3, 184",
            "st.d $r24, $r3, 192",
            "st.d $r25, $r3, 200",
            "st.d $r26, $r3, 208",
            "st.d $r27, $r3, 216",
            "st.d $r28, $r3, 224",
            "st.d $r29, $r3, 232",
            "st.d $r30, $r3, 240",
            "st.d $r31, $r3, 248",
            // save ERA
            "csrrd $r12, {LOONGARCH_CSR_ERA}",
            "st.d $r12, $r3, 256",

            // save GCSRS
            // "gcsrrd $r12, {LOONGARCH_GCSR_CRMD}",
            // "st.d $r12, $r3, 256+8*1",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PRMD}",
            // "st.d $r12, $r3, 256+8*2",
            // "gcsrrd $r12, {LOONGARCH_GCSR_EUEN}",
            // "st.d $r12, $r3, 256+8*3",
            // "gcsrrd $r12, {LOONGARCH_GCSR_MISC}",
            // "st.d $r12, $r3, 256+8*4",
            // "gcsrrd $r12, {LOONGARCH_GCSR_ECTL}",
            // "st.d $r12, $r3, 256+8*5",
            // "gcsrrd $r12, {LOONGARCH_GCSR_ESTAT}",
            // "st.d $r12, $r3, 256+8*6",
            // "gcsrrd $r12, {LOONGARCH_GCSR_ERA}",
            // "st.d $r12, $r3, 256+8*7",
            // "gcsrrd $r12, {LOONGARCH_GCSR_BADV}",
            // "st.d $r12, $r3, 256+8*8",
            // "gcsrrd $r12, {LOONGARCH_GCSR_BADI}",
            // "st.d $r12, $r3, 256+8*9",
            // "gcsrrd $r12, {LOONGARCH_GCSR_EENTRY}",
            // "st.d $r12, $r3, 256+8*10",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBIDX}",
            // "st.d $r12, $r3, 256+8*11",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBEHI}",
            // "st.d $r12, $r3, 256+8*12",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO0}",
            // "st.d $r12, $r3, 256+8*13",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO1}",
            // "st.d $r12, $r3, 256+8*14",
            // "gcsrrd $r12, {LOONGARCH_GCSR_ASID}",
            // "st.d $r12, $r3, 256+8*15",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PGDL}",
            // "st.d $r12, $r3, 256+8*16",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PGDH}",
            // "st.d $r12, $r3, 256+8*17",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PGD}",
            // "st.d $r12, $r3, 256+8*18",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PWCL}",
            // "st.d $r12, $r3, 256+8*19",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PWCH}",
            // "st.d $r12, $r3, 256+8*20",
            // "gcsrrd $r12, {LOONGARCH_GCSR_STLBPS}",
            // "st.d $r12, $r3, 256+8*21",
            // "gcsrrd $r12, {LOONGARCH_GCSR_RAVCFG}",
            // "st.d $r12, $r3, 256+8*22",
            // "gcsrrd $r12, {LOONGARCH_GCSR_CPUID}",
            // "st.d $r12, $r3, 256+8*23",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG1}",
            // "st.d $r12, $r3, 256+8*24",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG2}",
            // "st.d $r12, $r3, 256+8*25",
            // "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG3}",
            // "st.d $r12, $r3, 256+8*26",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE0}",
            // "st.d $r12, $r3, 256+8*27",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE1}",
            // "st.d $r12, $r3, 256+8*28",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE2}",
            // "st.d $r12, $r3, 256+8*29",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE3}",
            // "st.d $r12, $r3, 256+8*30",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE4}",
            // "st.d $r12, $r3, 256+8*31",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE5}",
            // "st.d $r12, $r3, 256+8*32",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE6}",
            // "st.d $r12, $r3, 256+8*33",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE7}",
            // "st.d $r12, $r3, 256+8*34",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE8}",
            // "st.d $r12, $r3, 256+8*35",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE9}",
            // "st.d $r12, $r3, 256+8*36",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE10}",
            // "st.d $r12, $r3, 256+8*37",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE11}",
            // "st.d $r12, $r3, 256+8*38",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE12}",
            // "st.d $r12, $r3, 256+8*39",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE13}",
            // "st.d $r12, $r3, 256+8*40",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE14}",
            // "st.d $r12, $r3, 256+8*41",
            // "gcsrrd $r12, {LOONGARCH_GCSR_SAVE15}",
            // "st.d $r12, $r3, 256+8*42",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TID}",
            // "st.d $r12, $r3, 256+8*43",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TCFG}",
            // "st.d $r12, $r3, 256+8*44",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TVAL}",
            // "st.d $r12, $r3, 256+8*45",
            // "gcsrrd $r12, {LOONGARCH_GCSR_CNTC}",
            // "st.d $r12, $r3, 256+8*46",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TICLR}",
            // "st.d $r12, $r3, 256+8*47",
            // "gcsrrd $r12, {LOONGARCH_GCSR_LLBCTL}",
            // "st.d $r12, $r3, 256+8*48",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRENTRY}",
            // "st.d $r12, $r3, 256+8*49",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRBADV}",
            // "st.d $r12, $r3, 256+8*50",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRERA}",
            // "st.d $r12, $r3, 256+8*51",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRSAVE}",
            // "st.d $r12, $r3, 256+8*52",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO0}",
            // "st.d $r12, $r3, 256+8*53",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO1}",
            // "st.d $r12, $r3, 256+8*54",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBREHI}",
            // "st.d $r12, $r3, 256+8*55",
            // "gcsrrd $r12, {LOONGARCH_GCSR_TLBRPRMD}",
            // "st.d $r12, $r3, 256+8*56",
            // "gcsrrd $r12, {LOONGARCH_GCSR_DMW0}",
            // "st.d $r12, $r3, 256+8*57",
            // "gcsrrd $r12, {LOONGARCH_GCSR_DMW1}",
            // "st.d $r12, $r3, 256+8*58",
            // "gcsrrd $r12, {LOONGARCH_GCSR_DMW2}",
            // "st.d $r12, $r3, 256+8*59",
            // "gcsrrd $r12, {LOONGARCH_GCSR_DMW3}",
            // "st.d $r12, $r3, 256+8*60",
            // // now let's save the zone's pgd to ZoneContext
            // "csrrd $r12, {LOONGARCH_CSR_PGDL}",
            // "st.d $r12, $r3, 256+8*61", // PGDL
            // "csrrd $r13, {LOONGARCH_CSR_PGDH}",
            // "st.d $r13, $r3, 256+8*62", // PGDH
            // // now let's switch KSAVE5 and KSAVE6, which should already
            // // be set to kernel's pagetable base
            // "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
            // "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
            // "csrwr $r12, {LOONGARCH_CSR_PGDL}",
            // "csrwr $r13, {LOONGARCH_CSR_PGDH}",
            // "invtlb 0, $r0, $r0",
            // save $r3 (previously saved in DESAVE) this is guest sp
            "csrrd $r12, {LOONGARCH_CSR_DESAVE}",
            "st.d $r12, $r3, 24",
            // $r3 -> a0, now the param of zone_trap_handler is ok
            "move $r4, $r3",
            // rewind sp to PerCpu default stack top from KSAVE4
            "csrrd $r3, {LOONGARCH_CSR_SAVE4}",
            "bl trap_handler",
            LOONGARCH_CSR_SAVE3 = const 0x33,
            LOONGARCH_CSR_SAVE4 = const 0x34,
            LOONGARCH_CSR_DESAVE = const 0x502,
            LOONGARCH_CSR_ERA = const 0x6,
            // LOONGARCH_GCSR_CRMD = const 0x0,
            // LOONGARCH_GCSR_PRMD = const 0x1,
            // LOONGARCH_GCSR_EUEN = const 0x2,
            // LOONGARCH_GCSR_MISC = const 0x3,
            // LOONGARCH_GCSR_ECTL = const 0x4,
            // LOONGARCH_GCSR_ESTAT = const 0x5,
            // LOONGARCH_GCSR_ERA = const 0x6,
            // LOONGARCH_GCSR_BADV = const 0x7,
            // LOONGARCH_GCSR_BADI = const 0x8,
            // LOONGARCH_GCSR_EENTRY = const 0xc,
            // LOONGARCH_GCSR_TLBIDX = const 0x10,
            // LOONGARCH_GCSR_TLBEHI = const 0x11,
            // LOONGARCH_GCSR_TLBELO0 = const 0x12,
            // LOONGARCH_GCSR_TLBELO1 = const 0x13,
            // LOONGARCH_GCSR_ASID = const 0x18,
            // LOONGARCH_GCSR_PGDL = const 0x19,
            // LOONGARCH_GCSR_PGDH = const 0x1a,
            // LOONGARCH_GCSR_PGD = const 0x1b,
            // LOONGARCH_GCSR_PWCL = const 0x1c,
            // LOONGARCH_GCSR_PWCH = const 0x1d,
            // LOONGARCH_GCSR_STLBPS = const 0x1e,
            // LOONGARCH_GCSR_RAVCFG = const 0x1f,
            // LOONGARCH_GCSR_CPUID = const 0x20,
            // LOONGARCH_GCSR_PRCFG1 = const 0x21,
            // LOONGARCH_GCSR_PRCFG2 = const 0x22,
            // LOONGARCH_GCSR_PRCFG3 = const 0x23,
            // LOONGARCH_GCSR_SAVE0 = const 0x30,
            // LOONGARCH_GCSR_SAVE1 = const 0x31,
            // LOONGARCH_GCSR_SAVE2 = const 0x32,
            // LOONGARCH_GCSR_SAVE3 = const 0x33,
            // LOONGARCH_GCSR_SAVE4 = const 0x34,
            // LOONGARCH_GCSR_SAVE5 = const 0x35,
            // LOONGARCH_GCSR_SAVE6 = const 0x36,
            // LOONGARCH_GCSR_SAVE7 = const 0x37,
            // LOONGARCH_GCSR_SAVE8 = const 0x38,
            // LOONGARCH_GCSR_SAVE9 = const 0x39,
            // LOONGARCH_GCSR_SAVE10 = const 0x3a,
            // LOONGARCH_GCSR_SAVE11 = const 0x3b,
            // LOONGARCH_GCSR_SAVE12 = const 0x3c,
            // LOONGARCH_GCSR_SAVE13 = const 0x3d,
            // LOONGARCH_GCSR_SAVE14 = const 0x3e,
            // LOONGARCH_GCSR_SAVE15 = const 0x3f,
            // LOONGARCH_GCSR_TID = const 0x40,
            // LOONGARCH_GCSR_TCFG = const 0x41,
            // LOONGARCH_GCSR_TVAL = const 0x42,
            // LOONGARCH_GCSR_CNTC = const 0x43,
            // LOONGARCH_GCSR_TICLR = const 0x44,
            // LOONGARCH_GCSR_LLBCTL = const 0x60,
            // LOONGARCH_GCSR_TLBRENTRY = const 0x88,
            // LOONGARCH_GCSR_TLBRBADV = const 0x89,
            // LOONGARCH_GCSR_TLBRERA = const 0x8a,
            // LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
            // LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
            // LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
            // LOONGARCH_GCSR_TLBREHI = const 0x8e,
            // LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
            // LOONGARCH_GCSR_DMW0 = const 0x180,
            // LOONGARCH_GCSR_DMW1 = const 0x181,
            // LOONGARCH_GCSR_DMW2 = const 0x182,
            // LOONGARCH_GCSR_DMW3 = const 0x183,
            // // LOONGARCH_CSR_PGDL = const 0x19,
            // LOONGARCH_CSR_PGDH = const 0x1a,
            // LOONGARCH_CSR_SAVE5 = const 0x35,
            // LOONGARCH_CSR_SAVE6 = const 0x36,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn _hyp_trap_return(ctx: usize) {
    unsafe {
        asm!(
            // a0 -> sp
            "move  $r3, $r4",
            // restore ERA
            "ld.d $r12, $r3, 256",
            "csrwr $r12, {LOONGARCH_CSR_ERA}",
            // restore GCSRS
            // "ld.d $r12, $r3, 256+8*1",
            // "gcsrwr $r12, {LOONGARCH_GCSR_CRMD}",
            // "ld.d $r12, $r3, 256+8*2",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PRMD}",
            // "ld.d $r12, $r3, 256+8*3",
            // "gcsrwr $r12, {LOONGARCH_GCSR_EUEN}",
            // "ld.d $r12, $r3, 256+8*4",
            // "gcsrwr $r12, {LOONGARCH_GCSR_MISC}",
            // "ld.d $r12, $r3, 256+8*5",
            // "gcsrwr $r12, {LOONGARCH_GCSR_ECTL}",
            // "ld.d $r12, $r3, 256+8*6",
            // "gcsrwr $r12, {LOONGARCH_GCSR_ESTAT}",
            // "ld.d $r12, $r3, 256+8*7",
            // "gcsrwr $r12, {LOONGARCH_GCSR_ERA}",
            // "ld.d $r12, $r3, 256+8*8",
            // "gcsrwr $r12, {LOONGARCH_GCSR_BADV}",
            // "ld.d $r12, $r3, 256+8*9",
            // "gcsrwr $r12, {LOONGARCH_GCSR_BADI}",
            // "ld.d $r12, $r3, 256+8*10",
            // "gcsrwr $r12, {LOONGARCH_GCSR_EENTRY}",
            // "ld.d $r12, $r3, 256+8*11",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBIDX}",
            // "ld.d $r12, $r3, 256+8*12",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBEHI}",
            // "ld.d $r12, $r3, 256+8*13",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO0}",
            // "ld.d $r12, $r3, 256+8*14",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO1}",
            // "ld.d $r12, $r3, 256+8*15",
            // "gcsrwr $r12, {LOONGARCH_GCSR_ASID}",
            // "ld.d $r12, $r3, 256+8*16",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PGDL}",
            // "ld.d $r12, $r3, 256+8*17",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PGDH}",
            // "ld.d $r12, $r3, 256+8*18",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PGD}",
            // "ld.d $r12, $r3, 256+8*19",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PWCL}",
            // "ld.d $r12, $r3, 256+8*20",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PWCH}",
            // "ld.d $r12, $r3, 256+8*21",
            // "gcsrwr $r12, {LOONGARCH_GCSR_STLBPS}",
            // "ld.d $r12, $r3, 256+8*22",
            // "gcsrwr $r12, {LOONGARCH_GCSR_RAVCFG}",
            // "ld.d $r12, $r3, 256+8*23",
            // "gcsrwr $r12, {LOONGARCH_GCSR_CPUID}",
            // "ld.d $r12, $r3, 256+8*24",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG1}",
            // "ld.d $r12, $r3, 256+8*25",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG2}",
            // "ld.d $r12, $r3, 256+8*26",
            // "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG3}",
            // "ld.d $r12, $r3, 256+8*27",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE0}",
            // "ld.d $r12, $r3, 256+8*28",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE1}",
            // "ld.d $r12, $r3, 256+8*29",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE2}",
            // "ld.d $r12, $r3, 256+8*30",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE3}",
            // "ld.d $r12, $r3, 256+8*31",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE4}",
            // "ld.d $r12, $r3, 256+8*32",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE5}",
            // "ld.d $r12, $r3, 256+8*33",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE6}",
            // "ld.d $r12, $r3, 256+8*34",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE7}",
            // "ld.d $r12, $r3, 256+8*35",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE8}",
            // "ld.d $r12, $r3, 256+8*36",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE9}",
            // "ld.d $r12, $r3, 256+8*37",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE10}",
            // "ld.d $r12, $r3, 256+8*38",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE11}",
            // "ld.d $r12, $r3, 256+8*39",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE12}",
            // "ld.d $r12, $r3, 256+8*40",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE13}",
            // "ld.d $r12, $r3, 256+8*41",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE14}",
            // "ld.d $r12, $r3, 256+8*42",
            // "gcsrwr $r12, {LOONGARCH_GCSR_SAVE15}",
            // "ld.d $r12, $r3, 256+8*43",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TID}",
            // "ld.d $r12, $r3, 256+8*44",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TCFG}",
            // "ld.d $r12, $r3, 256+8*45",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TVAL}",
            // "ld.d $r12, $r3, 256+8*46",
            // "gcsrwr $r12, {LOONGARCH_GCSR_CNTC}",
            // "ld.d $r12, $r3, 256+8*47",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TICLR}",
            // "ld.d $r12, $r3, 256+8*48",
            // "gcsrwr $r12, {LOONGARCH_GCSR_LLBCTL}",
            // "ld.d $r12, $r3, 256+8*49",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRENTRY}",
            // "ld.d $r12, $r3, 256+8*50",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRBADV}",
            // "ld.d $r12, $r3, 256+8*51",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRERA}",
            // "ld.d $r12, $r3, 256+8*52",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRSAVE}",
            // "ld.d $r12, $r3, 256+8*53",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO0}",
            // "ld.d $r12, $r3, 256+8*54",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO1}",
            // "ld.d $r12, $r3, 256+8*55",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBREHI}",
            // "ld.d $r12, $r3, 256+8*56",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TLBRPRMD}",
            // "ld.d $r12, $r3, 256+8*57",
            // "gcsrwr $r12, {LOONGARCH_GCSR_DMW0}",
            // "ld.d $r12, $r3, 256+8*58",
            // "gcsrwr $r12, {LOONGARCH_GCSR_DMW1}",
            // "ld.d $r12, $r3, 256+8*59",
            // "gcsrwr $r12, {LOONGARCH_GCSR_DMW2}",
            // "ld.d $r12, $r3, 256+8*60",
            // "gcsrwr $r12, {LOONGARCH_GCSR_DMW3}",
            LOONGARCH_CSR_ERA = const 0x6,
            // LOONGARCH_GCSR_CRMD = const 0x0,
            // LOONGARCH_GCSR_PRMD = const 0x1,
            // LOONGARCH_GCSR_EUEN = const 0x2,
            // LOONGARCH_GCSR_MISC = const 0x3,
            // LOONGARCH_GCSR_ECTL = const 0x4,
            // LOONGARCH_GCSR_ESTAT = const 0x5,
            // LOONGARCH_GCSR_ERA = const 0x6,
            // LOONGARCH_GCSR_BADV = const 0x7,
            // LOONGARCH_GCSR_BADI = const 0x8,
            // LOONGARCH_GCSR_EENTRY = const 0xc,
            // LOONGARCH_GCSR_TLBIDX = const 0x10,
            // LOONGARCH_GCSR_TLBEHI = const 0x11,
            // LOONGARCH_GCSR_TLBELO0 = const 0x12,
            // LOONGARCH_GCSR_TLBELO1 = const 0x13,
            // LOONGARCH_GCSR_ASID = const 0x18,
            // LOONGARCH_GCSR_PGDL = const 0x19,
            // LOONGARCH_GCSR_PGDH = const 0x1a,
            // LOONGARCH_GCSR_PGD = const 0x1b,
            // LOONGARCH_GCSR_PWCL = const 0x1c,
            // LOONGARCH_GCSR_PWCH = const 0x1d,
            // LOONGARCH_GCSR_STLBPS = const 0x1e,
            // LOONGARCH_GCSR_RAVCFG = const 0x1f,
            // LOONGARCH_GCSR_CPUID = const 0x20,
            // LOONGARCH_GCSR_PRCFG1 = const 0x21,
            // LOONGARCH_GCSR_PRCFG2 = const 0x22,
            // LOONGARCH_GCSR_PRCFG3 = const 0x23,
            // LOONGARCH_GCSR_SAVE0 = const 0x30,
            // LOONGARCH_GCSR_SAVE1 = const 0x31,
            // LOONGARCH_GCSR_SAVE2 = const 0x32,
            // LOONGARCH_GCSR_SAVE3 = const 0x33,
            // LOONGARCH_GCSR_SAVE4 = const 0x34,
            // LOONGARCH_GCSR_SAVE5 = const 0x35,
            // LOONGARCH_GCSR_SAVE6 = const 0x36,
            // LOONGARCH_GCSR_SAVE7 = const 0x37,
            // LOONGARCH_GCSR_SAVE8 = const 0x38,
            // LOONGARCH_GCSR_SAVE9 = const 0x39,
            // LOONGARCH_GCSR_SAVE10 = const 0x3a,
            // LOONGARCH_GCSR_SAVE11 = const 0x3b,
            // LOONGARCH_GCSR_SAVE12 = const 0x3c,
            // LOONGARCH_GCSR_SAVE13 = const 0x3d,
            // LOONGARCH_GCSR_SAVE14 = const 0x3e,
            // LOONGARCH_GCSR_SAVE15 = const 0x3f,
            // LOONGARCH_GCSR_TID = const 0x40,
            // LOONGARCH_GCSR_TCFG = const 0x41,
            // LOONGARCH_GCSR_TVAL = const 0x42,
            // LOONGARCH_GCSR_CNTC = const 0x43,
            // LOONGARCH_GCSR_TICLR = const 0x44,
            // LOONGARCH_GCSR_LLBCTL = const 0x60,
            // LOONGARCH_GCSR_TLBRENTRY = const 0x88,
            // LOONGARCH_GCSR_TLBRBADV = const 0x89,
            // LOONGARCH_GCSR_TLBRERA = const 0x8a,
            // LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
            // LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
            // LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
            // LOONGARCH_GCSR_TLBREHI = const 0x8e,
            // LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
            // LOONGARCH_GCSR_DMW0 = const 0x180,
            // LOONGARCH_GCSR_DMW1 = const 0x181,
            // LOONGARCH_GCSR_DMW2 = const 0x182,
            // LOONGARCH_GCSR_DMW3 = const 0x183,
        );
        // asm!(
        //   // vm-pagetable -> save5 and save6
        //   "ld.d $r12, $r3, 256+8*61",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "ld.d $r12, $r3, 256+8*62",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE6}",
        //   // kernel-pagetable -> r12 and r13
        //   "csrrd $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrrd $r13, {LOONGARCH_CSR_PGDH}",
        //   // kernel_pagetable -> save5 and save6
        //   // old save5/save6(vm_pagetable) -> r12/r13
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
        //   // change pagetable from kernel pagetable to vm page table
        //   "csrwr $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrwr $r13, {LOONGARCH_CSR_PGDH}",
        //   "invtlb 0, $r0, $r0",
        //   LOONGARCH_CSR_SAVE5 = const 0x35,
        //   LOONGARCH_CSR_SAVE6 = const 0x36,
        //   LOONGARCH_CSR_PGDL = const 0x19,
        //   LOONGARCH_CSR_PGDH = const 0x1a,
        // );
        asm!(
            // restore sp
            "ld.d $r12, $r3, 24",
            "csrwr $r12, {LOONGARCH_CSR_DESAVE}",
            // restore 32 GPRS:
            "ld.d $r0, $r3, 0",
            "ld.d $r1, $r3, 8",
            "ld.d $r2, $r3, 16",
            //ld.d $r3, $r3, 24
            "ld.d $r4, $r3, 32",
            "ld.d $r5, $r3, 40",
            "ld.d $r6, $r3, 48",
            "ld.d $r7, $r3, 56",
            "ld.d $r8, $r3, 64",
            "ld.d $r9, $r3, 72",
            "ld.d $r10, $r3, 80",
            "ld.d $r11, $r3, 88",
            "ld.d $r12, $r3, 96",
            "ld.d $r13, $r3, 104",
            "ld.d $r14, $r3, 112",
            "ld.d $r15, $r3, 120",
            "ld.d $r16, $r3, 128",
            "ld.d $r17, $r3, 136",
            "ld.d $r18, $r3, 144",
            "ld.d $r19, $r3, 152",
            "ld.d $r20, $r3, 160",
            "ld.d $r21, $r3, 168",
            "ld.d $r22, $r3, 176",
            "ld.d $r23, $r3, 184",
            "ld.d $r24, $r3, 192",
            "ld.d $r25, $r3, 200",
            "ld.d $r26, $r3, 208",
            "ld.d $r27, $r3, 216",
            "ld.d $r28, $r3, 224",
            "ld.d $r29, $r3, 232",
            "ld.d $r30, $r3, 240",
            "ld.d $r31, $r3, 248",
            "csrwr $r3, {LOONGARCH_CSR_DESAVE}",
            "ertn",
            LOONGARCH_CSR_DESAVE = const 0x502
        );
    }
}

/// call this before running any guests to acquire the default GCSR values
pub fn dump_reset_gcsrs() -> ZoneContext {
    let mut ctx = ZoneContext::new();
    ctx.gcsr_crmd = read_gcsr_crmd();
    ctx.gcsr_prmd = read_gcsr_prmd();
    ctx.gcsr_euen = read_gcsr_euen();
    ctx.gcsr_misc = read_gcsr_misc();
    ctx.gcsr_ectl = read_gcsr_ectl();
    ctx.gcsr_estat = read_gcsr_estat();
    ctx.gcsr_era = read_gcsr_era();
    ctx.gcsr_badv = read_gcsr_badv();
    ctx.gcsr_badi = read_gcsr_badi();
    ctx.gcsr_eentry = read_gcsr_eentry();
    ctx.gcsr_tlbidx = read_gcsr_tlbidx();
    ctx.gcsr_tlbehi = read_gcsr_tlbehi();
    ctx.gcsr_tlbelo0 = read_gcsr_tlbelo0();
    ctx.gcsr_tlbelo1 = read_gcsr_tlbelo1();
    ctx.gcsr_asid = read_gcsr_asid();
    ctx.gcsr_pgdl = read_gcsr_pgdl();
    ctx.gcsr_pgdh = read_gcsr_pgdh();
    ctx.gcsr_pgd = read_gcsr_pgd();
    ctx.gcsr_pwcl = read_gcsr_pwcl();
    ctx.gcsr_pwch = read_gcsr_pwch();
    ctx.gcsr_stlbps = read_gcsr_stlbps();
    ctx.gcsr_ravcfg = read_gcsr_ravcfg();
    ctx.gcsr_cpuid = read_gcsr_cpuid();
    ctx.gcsr_prcfg1 = read_gcsr_prcfg1();
    ctx.gcsr_prcfg2 = read_gcsr_prcfg2();
    ctx.gcsr_prcfg3 = read_gcsr_prcfg3();
    ctx.gcsr_save0 = read_gcsr_save0();
    ctx.gcsr_save1 = read_gcsr_save1();
    ctx.gcsr_save2 = read_gcsr_save2();
    ctx.gcsr_save3 = read_gcsr_save3();
    ctx.gcsr_save4 = read_gcsr_save4();
    ctx.gcsr_save5 = read_gcsr_save5();
    ctx.gcsr_save6 = read_gcsr_save6();
    ctx.gcsr_save7 = read_gcsr_save7();
    ctx.gcsr_save8 = read_gcsr_save8();
    ctx.gcsr_save9 = read_gcsr_save9();
    ctx.gcsr_save10 = read_gcsr_save10();
    ctx.gcsr_save11 = read_gcsr_save11();
    ctx.gcsr_save12 = read_gcsr_save12();
    ctx.gcsr_save13 = read_gcsr_save13();
    ctx.gcsr_save14 = read_gcsr_save14();
    ctx.gcsr_save15 = read_gcsr_save15();
    ctx.gcsr_tid = read_gcsr_tid();
    ctx.gcsr_tcfg = read_gcsr_tcfg();
    ctx.gcsr_tval = read_gcsr_tval();
    ctx.gcsr_cntc = read_gcsr_cntc();
    ctx.gcsr_ticlr = read_gcsr_ticlr();
    ctx.gcsr_llbctl = read_gcsr_llbctl();
    ctx.gcsr_tlbrentry = read_gcsr_tlbrentry();
    ctx.gcsr_tlbrbadv = read_gcsr_tlbrbadv();
    ctx.gcsr_tlbrera = read_gcsr_tlbrera();
    ctx.gcsr_tlbrsave = read_gcsr_tlbrsave();
    ctx.gcsr_tlbrelo0 = read_gcsr_tlbrrelo0();
    ctx.gcsr_tlbrelo1 = read_gcsr_tlbrrelo1();
    ctx.gcsr_tlbrehi = read_gcsr_tlbrrehi();
    ctx.gcsr_tlbrprmd = read_gcsr_tlbrprmd();
    ctx.gcsr_dmw0 = read_gcsr_dmw0();
    ctx.gcsr_dmw1 = read_gcsr_dmw1();
    ctx.gcsr_dmw2 = read_gcsr_dmw2();
    ctx.gcsr_dmw3 = read_gcsr_dmw3();

    ctx
}

fn extract_field(inst: usize, offset: usize, length: usize) -> usize {
    let mask = (1 << length) - 1;
    (inst >> offset) & mask
}

/// get the sign-extended imm12 to i64
fn imm12toi64(imm12: usize) -> isize {
    let imm12 = imm12 as isize;
    let imm12 = imm12 << 52;
    imm12 >> 52
}

const INT_IPI: usize = 12;
const IPI_BIT: usize = 1 << 12;
const TIMER_BIT: usize = 1 << 11;
const HWI0: usize = 1 << 2;
const HWI1: usize = 1 << 3;
const HWI2: usize = 1 << 4;
const HWI3: usize = 1 << 5;
const HWI4: usize = 1 << 6;
const HWI5: usize = 1 << 7;
const HWI6: usize = 1 << 8;
const HWI7: usize = 1 << 9;
const SWI0: usize = 1 << 0;
const SWI1: usize = 1 << 1;

fn do_deliver_irq(irq_flags: usize, clear_flag: bool) {
    for irq in (0..13).rev() {
        let mask = 1 << irq;
        if irq_flags & mask != 0 {
            if clear_flag {
                // clear irq
                clear_irq(irq, false); // para is_hardware is invalid here
            } else {
                // inject irq
                inject_irq(irq, false);
            }
        }
    }
}

fn deliver_irq() {
    let pcpu_id = this_cpu_id();
    let pcpu_data = get_cpu_data(pcpu_id);
    let irq_pending = pcpu_data.arch_cpu.irq_pending;
    let irq_clear = pcpu_data.arch_cpu.irq_clear;
    if irq_pending == 0 && irq_clear == 0 {
        return;
    }
    assert!(irq_clear & irq_pending == 0);

    do_deliver_irq(irq_clear, true);
    do_deliver_irq(irq_pending, false);
    pcpu_data.arch_cpu.irq_pending = 0;
    pcpu_data.arch_cpu.irq_clear = 0;
}

/// handle loongarch64 interrupts here
fn handle_interrupt(is: usize) {
    // Handle IPI interrupts
    let pcpu_id_this: usize = this_cpu_id();
    let pcpu_data = get_cpu_data(pcpu_id_this);

    if is & IPI_BIT != 0 {
        let ipi_status = get_ipi_status(pcpu_id_this);

        let mut ipistate = pcpu_data.arch_cpu.ipi_state.lock();
        let pcpu_ipi_status = ipistate.status as usize; // read

        reset_ipi(pcpu_id_this); // clear

        if pcpu_ipi_status & SMP_BOOT_CPU != 0 {
            if pcpu_data.arch_cpu.power_on == true {
                panic!(
                    "pcpu : {} has already power on, this should not happen",
                    pcpu_id_this
                );
            }
            // this should be done by firmware, but we do this here, because linux kernel does not do it
            ipistate.status &= !(ipi_status as u32);

            let first_pcpu_id = pcpu_data
                .zone
                .as_ref()
                .unwrap()
                .read()
                .cpu_set()
                .first_cpu()
                .unwrap();

            if (first_pcpu_id == pcpu_id_this) {
                // this is the first cpu in the zone
                drop(ipistate); // remember! avoid deadlock
                pcpu_data.arch_cpu.run();
                panic!("can't reach here");
            } else {
                // this is not the first cpu in the zone, read smpboot_entry from ipistate.buf
                // let smpboot_entry = ipistate.buf[first_pcpu_id] as usize;
                let smpboot_entry = ipistate.buf[0] as usize;
                // note!, always fetch smpboot_entry from cpu[0]! boneinscri 2026.04
                warn!("pcpu_ipi_status = {:#x}, first_pcpu_id = {:#x}, smpboot_entry: {:#x}, pcpu_ipi_status = {:#x}", 
                pcpu_ipi_status, first_pcpu_id, smpboot_entry, ipistate.status as usize);
                drop(ipistate); // remember! avoid deadlock
                pcpu_data.arch_cpu.run_secondary(smpboot_entry);
                panic!("can't reach here");
            }
        } else if pcpu_ipi_status & HVISOR_SHUTDOWN != 0 {
            // if pcpu_data.arch_cpu.power_on == false {
            //     panic!("pcpu : {} has not power on, this should not happen", pcpu_id_this);
            // }
            ipistate.status &= !(ipi_status as u32);
            drop(ipistate);
            pcpu_data.arch_cpu.idle();
        } else if pcpu_ipi_status & HVISOR_EVENT_VIRTIO_INJECT_IRQ != 0 {
            if pcpu_data.arch_cpu.power_on == false {
                panic!(
                    "pcpu : {} has not power on, this should not happen",
                    pcpu_id_this
                );
            }
            ipistate.status &= !(ipi_status as u32);
            drop(ipistate);
            handle_virtio_irq();
        } else if pcpu_ipi_status & HVISOR_EVENT_WAKEUP_VIRTIO_DEVICE != 0 {
            panic!("HVISOR_EVENT_WAKEUP_VIRTIO_DEVICE, not tested");
        } else if pcpu_ipi_status & HVISOR_EVENT_VIRTIO_CLEAR_IRQ != 0 {
            panic!("HVISOR_EVENT_VIRTIO_CLEAR_IRQ, not tested");
        } else if pcpu_ipi_status != 0 {
            drop(ipistate);
            pcpu_data.arch_cpu.add_irq(INT_IPI);
        } else {
        }
        return;
    }

    // Handle timer interrupts
    if is & TIMER_BIT != 0 {
        warn!("Timer interrupt received");
        loongArch64::register::ticlr::clear_timer_interrupt();
        crate::device::irqchip::ls7a2000::clear_hwi_injected_irq();
        return;
    }

    // Handle hardware interrupts (HWI)
    let hwi_mask = HWI0 | HWI1 | HWI2 | HWI3 | HWI4 | HWI5 | HWI6 | HWI7;
    if is & hwi_mask != 0 {
        let cpu_id = this_cpu_id();
        let sr = get_extioi_sr();
        warn!(
            "CPU {} received HWI interrupt, status = {:#x}, extioi status: {}",
            cpu_id, is, sr
        );
        return;
    }

    if is & SWI0 != 0 {
        panic!("swi0 not handled");
    }
    if is & SWI1 != 0 {
        panic!("swi1 not handled");
    }

    // Handle unknown interrupts
    error!("Received unhandled interrupt, status = {:#x}", is);
}

/// hypercall handler
fn handle_hvc(ctx: &mut ZoneContext) {
    // HVC
    let code = ctx.get_a0();
    let arg0 = ctx.get_a1();
    let arg1 = ctx.get_a2();

    debug!(
        "HVC exception, HVC call code: {:#x}, arg0: {:#x}, arg1: {:#x}",
        code, arg0, arg1
    );
    let cpu_data = this_cpu_data();
    let res = match HyperCall::new(cpu_data).hypercall(code as _, arg0 as _, arg1 as _) {
        Ok(ret) => ret as _,
        Err(e) => {
            error!("HVC exception failed: {:?}", e);
            e.code()
        }
    };
    debug!("HVC result: {:#x}", res);
    ctx.set_a0(res as _);
    ctx.sepc += 4;
}

fn emulate_cpucfg(ins: usize, ctx: &mut ZoneContext) {
    // cpucfg
    // now let get rd and rj, cpucfg rd[4:0], rj[9:5]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let cpucfg_target_idx = ctx.x[rj];
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let cpucfg_target_idx = ctx.x[rj];

    const MAX_CPUCFG_REGS: usize = 21;

    // info!(
    //     "cpucfg emulation, target cpucfg index is {:#x}",
    //     cpucfg_target_idx
    // );

    if cpucfg_target_idx >= MAX_CPUCFG_REGS {
        // invalid cpucfg target
        warn!("invalid cpucfg target");
        ctx.x[rd] = 0;
        // according to manual, we should set result to 0 if index is invalid
    } else {
        // just run cpucfg here
        let mut result = 0;
        unsafe {
            asm!("cpucfg {}, {}", out(reg) result, in(reg) cpucfg_target_idx);
        }
        if cpucfg_target_idx == 0x2 {
            result &= !(1 << 10); // shutdown lvz of vm -- boneinscri 2026.04
        }
        ctx.x[rd] = result;
        // finish the emulation by tweaking the ZoneContext's registers
        // as ctx.sepc is already added by 4 which means we will jump to next instruction - wheatfox
    }
}

// modified -- boneinscri 2026.04
fn emulate_csrx(ins: usize, ctx: &mut ZoneContext) {
    // csrrd csrwr csrxchg
    // let ty = (ins >> 5) & 0x1f;
    // let rd = ins & 0x1f;
    // let csr = (ins >> 10) & 0x3fff;
    let rj = extract_field(ins, 5, 5);
    let rd = extract_field(ins, 0, 5);
    let csr_id = extract_field(ins, 10, 14);
    // ty: [9:5], 0 - csrrd, 1 - csrwr, else - csrxchg
    // rd [4:0]
    // csr [23:10] 14 bits
    assert!(csr_id <= 0x502);

    let pcpu_data_this = this_cpu_data();
    //  TODO: pay attention to PERFCTRL0
    match rj {
        0 => {
            // csrrd
            let val = pcpu_data_this.arch_cpu.csr[csr_id];
            ctx.x[rd] = val;
            // info!("csrrd emulation for CSR {:#x}, r val = {:#x}", csr_id, val);
        }
        1 => {
            // csrwr
            let val = ctx.x[rd];
            // info!("csrwr emulation for CSR {:#x}, w val = {:#x}", csr_id, val);
            pcpu_data_this.arch_cpu.csr[csr_id] = val;
            ctx.x[rd] = val;
        }
        _ => {
            // csrxchg
            // info!("csrxchg emulation for CSR {:#x}, val : {:#x}, csr_mask : {:#x}",
            //     csr_id, ctx.x[rd], ctx.x[rj]);
            let mut val = ctx.x[rd];
            let csr_mask = ctx.x[rj];
            let mut old = pcpu_data_this.arch_cpu.csr[csr_id]; // read old value from sw csr
            val = (old & !csr_mask) | (val & csr_mask);
            pcpu_data_this.arch_cpu.csr[csr_id] = val; // record the new value from trap ctx
            old = old & csr_mask;
            ctx.x[rd] = old; // return old value to guest
        }
    }
}

fn emulate_cacop(ins: usize, ctx: &mut ZoneContext) {
    // cacop code,rj,si12   0000011000 si12 rj[9:5] code[4:0]
    warn!("cacop emulation not implemented, skipped this instruction");
}

fn emulate_idle(ins: usize, ctx: &mut ZoneContext) {
    // idle level           0000011001 0010001 level[14:0]
    let level = extract_field(ins, 0, 15);
    trace!("guest request an idle at level {:#x}", level);
}

fn ty2str(ty: usize) -> &'static str {
    match ty {
        0 => "iocsrrd.b",
        1 => "iocsrrd.h",
        2 => "iocsrrd.w",
        3 => "iocsrrd.d",
        4 => "iocsrwr.b",
        5 => "iocsrwr.h",
        6 => "iocsrwr.w",
        7 => "iocsrwr.d",
        _ => "unknown",
    }
}

// boneinscri 2026.04
pub fn loongarch_iocsr_read(pcpu_id: usize, addr: usize, len: usize) -> usize {
    let iocsr_type = get_iocsr_type(addr);
    match iocsr_type {
        IOCSR_TYPE_IPI => {
            // IPI
            let ret = loongarch_ipi_readl(pcpu_id, addr, len);
            ret
        }
        IOCSR_TYPE_EIOINTC => {
            // EIOINTC
            let ret = loongarch_eiointc_readl(pcpu_id, addr, len);
            ret
        }
        IOCSR_TYPE_EIOINTC_VIRT => {
            panic!("EIOINTC_VIRT detected, this is not supported yet");
        }
        _ => {
            let mut addr_real = addr;
            do_real_read_iocsr(addr_real, len)
        }
    }
}
pub fn loongarch_iocsr_write(pcpu_id: usize, addr: usize, val: usize, len: usize) -> usize {
    let iocsr_type = get_iocsr_type(addr);
    match iocsr_type {
        IOCSR_TYPE_IPI => {
            // IPI
            let ret = loongarch_ipi_writel(pcpu_id, addr, val, len);
            ret
        }
        IOCSR_TYPE_EIOINTC => {
            // EIOINTC
            let ret = loongarch_eiointc_writel(pcpu_id, addr, val, len);
            ret
        }
        IOCSR_TYPE_EIOINTC_VIRT => {
            panic!("EIOINTC_VIRT detected, this is not supported yet");
        }
        _ => {
            let mut addr_real = addr;
            do_real_write_iocsr(addr_real, val, len);
            0
        }
    }
}

fn emulate_iocsr(ins: usize, ctx: &mut ZoneContext) {
    // iocsrrd.b rd, rj     0000011001 001000000000 rj[9:5] rd[4:0]
    // iocsrrd.h rd, rj     0000011001 001000000001 rj[9:5] rd[4:0]
    // iocsrrd.w rd, rj     0000011001 001000000010 rj[9:5] rd[4:0]
    // iocsrrd.d rd, rj     0000011001 001000000011 rj[9:5] rd[4:0]
    // iocsrwr.b rd, rj     0000011001 001000000100 rj[9:5] rd[4:0]
    // iocsrwr.h rd, rj     0000011001 001000000101 rj[9:5] rd[4:0]
    // iocsrwr.w rd, rj     0000011001 001000000110 rj[9:5] rd[4:0]
    // iocsrwr.d rd, rj     0000011001 001000000111 rj[9:5] rd[4:0]
    let ty = extract_field(ins, 10, 3);
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    debug!("iocsr emulation, ty = {}, rd = {}, rj = {}", ty, rd, rj);
    debug!("GPR[rd] = {:#x}, GPR[rj] = {:#x}", ctx.x[rd], ctx.x[rj]);

    let mut len = 0;
    let mut is_write = false;
    let addr = ctx.x[rj] as usize;
    let val = ctx.x[rd] as usize;

    if ty < 8 {
        len = 1 << (ty % 4); // 0-3:1,2,4,8; 4-7:1,2,4,8
        is_write = ty >= 4;
    } else {
        panic!("emulate_iocsr, invalid iocsr type, this is impossible");
    }

    // TODO : modify to vCPU
    let pcpu_id_this = this_cpu_id();

    if is_write {
        let ret = loongarch_iocsr_write(pcpu_id_this, addr, val, len);
        return;
    } else {
        let ret = loongarch_iocsr_read(pcpu_id_this, addr, len);
        ctx.x[rd] = ret;
        return;
    }
}

fn emulate_iocsr_legacy(ins: usize, ctx: &mut ZoneContext) {
    // iocsrrd.b rd, rj     0000011001 001000000000 rj[9:5] rd[4:0]
    // iocsrrd.h rd, rj     0000011001 001000000001 rj[9:5] rd[4:0]
    // iocsrrd.w rd, rj     0000011001 001000000010 rj[9:5] rd[4:0]
    // iocsrrd.d rd, rj     0000011001 001000000011 rj[9:5] rd[4:0]
    // iocsrwr.b rd, rj     0000011001 001000000100 rj[9:5] rd[4:0]
    // iocsrwr.h rd, rj     0000011001 001000000101 rj[9:5] rd[4:0]
    // iocsrwr.w rd, rj     0000011001 001000000110 rj[9:5] rd[4:0]
    // iocsrwr.d rd, rj     0000011001 001000000111 rj[9:5] rd[4:0]
    // let ty = (ins >> 10) & 0x7;
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    let ty = extract_field(ins, 10, 3);
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    debug!("iocsr emulation, ty = {}, rd = {}, rj = {}", ty, rd, rj);
    debug!("GPR[rd] = {:#x}, GPR[rj] = {:#x}", ctx.x[rd], ctx.x[rj]);
    match ty {
        0 => {
            // iocsrrd.b
            // GPR[rd] = iocsrrd.b(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.b {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xff;
        }
        1 => {
            // iocsrrd.h
            // GPR[rd] = iocsrrd.h(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.h {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xffff;
        }
        2 => {
            // iocsrrd.w
            // GPR[rd] = iocsrrd.w(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.w {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xffffffff;
        }
        3 => {
            // iocsrrd.d
            // GPR[rd] = iocsrrd.d(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.d {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val;
        }
        4 => {
            // iocsrwr.b
            // iocsrwr.b(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.b {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        5 => {
            // iocsrwr.h
            // iocsrwr.h(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.h {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        6 => {
            // iocsrwr.w
            // iocsrwr.w(GPR[rd], GPR[rj])

            // hack: since guest linux will use iocsrwr.w [xxx] 0x1014 to send IPI to itself (ACTION_IRQ_WORK)
            // we need to check if ctx.x[rj] is 0x1014, if so, we should parse ctx.x[rd][31:0]
            // then inject IPI bit to GCSR_ESTAT, and prepare the 7A2000's IPI register to have the exact target status
            // using the debug ANY_SEND register (TODO)

            let target_io = ctx.x[rj];
            let target_write_data = ctx.x[rd];

            match target_io {
                0x1014 => {
                    // IPI send issued from guest is tricky ...
                    // IPI_send is 32 bit, we ignore the upper 32 bits
                    // bit [31]: wait for completion
                    // bit [25:16] target cpu id
                    // bit [4:0] ipi id (IPI_status, 32 bit) indicates the IPI type (0-31)
                    let ipi_send = target_write_data as u32;
                    let ipi_id = ipi_send & 0x1f;
                    let target_cpu_id = (ipi_send >> 16) & 0x3ff;
                    let wait_for_completion = (ipi_send >> 31) & 0x1;
                    warn!("IPI send issued from guest, ipi_id = {:#x}, target_cpu_id = {:#x}, wait_for_completion = {:#x}", ipi_id, target_cpu_id, wait_for_completion);
                    if target_cpu_id == this_cpu_id() as u32 {
                        warn!("send IPI to itself, injecting IPI to GCSR_ESTAT");
                        inject_irq(INT_IPI, false);
                    } else {
                        // TODO
                        panic!("send IPI from guest to other cpu is not supported yet!");
                    }
                }
                _ => unsafe {
                    asm!("iocsrwr.w {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
                },
            }
        }
        7 => {
            // iocsrwr.d
            // iocsrwr.d(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.d {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        _ => {
            // should not reach here
            panic!("invalid iocsr type, this is impossible");
        }
    }
}

const UART0_BASE: usize = 0x1fe001e0;
const UART0_END: usize = 0x1fe001e8;

fn emulate_ld_b(ins: usize, ctx: &mut ZoneContext) {
    // ld.b   rd, rj, si12  opcode[31:22]=0010100000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff; ??? should be 0xfff
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);

    info!("ld.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

fn emulate_st_b(ins: usize, ctx: &mut ZoneContext) {
    // st.b   rd, rj, si12  opcode[31:22]=0010100100 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);
    // info!("st.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // store (paddr, BYTE, GR[rd])
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    // info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

fn emulate_ld_bu(ins: usize, ctx: &mut ZoneContext) {
    // ld.bu  rd, rj, si12  opcode[31:22]=0010101000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);
    // info!("ld.bu emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

fn check_op_type(inst: usize, opcode: usize, opcode_length: usize) -> bool {
    let mask = (1 << opcode_length) - 1;
    let shifted = inst >> (32 - opcode_length);
    (shifted & mask) == opcode
}

const OPCODE_CPUCFG: usize = 0b0000000000000000011011;
const OPCODE_CPUCFG_LENGTH: usize = 22;
const OPCODE_CACOP: usize = 0b0000011000;
const OPCODE_CACOP_LENGTH: usize = 10;
const OPCODE_IDLE: usize = 0b00000_11001_0010001;
const OPCODE_IDLE_LENGTH: usize = 17;
const OPCODE_CSRX: usize = 0b00000100;
const OPCODE_CSRX_LENGTH: usize = 8;
const OPCODE_IOCSR: usize = 0b00000_11001_001000000;
const OPCODE_IOCSR_LENGTH: usize = 19;
const OPCODE_LD_B: usize = 0b0010100000;
const OPCODE_LD_B_LENGTH: usize = 10;
const OPCODE_ST_B: usize = 0b0010100100;
const OPCODE_ST_B_LENGTH: usize = 10;
const OPCODE_LD_BU: usize = 0b0010101000;
const OPCODE_LD_BU_LENGTH: usize = 10;
type OpcodeHandler = fn(usize, &mut ZoneContext);

fn emulate_instruction(era: usize, ins: usize, ctx: &mut ZoneContext) {
    let pc = era;
    // after we emulate the instruction, we should jump to next instruction
    ctx.sepc = pc + 4;

    let opcodes = vec![
        (
            OPCODE_CPUCFG,
            OPCODE_CPUCFG_LENGTH,
            emulate_cpucfg as OpcodeHandler,
        ),
        (
            OPCODE_CACOP,
            OPCODE_CACOP_LENGTH,
            emulate_cacop as OpcodeHandler,
        ),
        (
            OPCODE_IDLE,
            OPCODE_IDLE_LENGTH,
            emulate_idle as OpcodeHandler,
        ),
        (
            OPCODE_CSRX,
            OPCODE_CSRX_LENGTH,
            emulate_csrx as OpcodeHandler,
        ),
        (
            OPCODE_IOCSR,
            OPCODE_IOCSR_LENGTH,
            emulate_iocsr as OpcodeHandler,
        ),
        (
            OPCODE_LD_B,
            OPCODE_LD_B_LENGTH,
            emulate_ld_b as OpcodeHandler,
        ),
        (
            OPCODE_ST_B,
            OPCODE_ST_B_LENGTH,
            emulate_st_b as OpcodeHandler,
        ),
        (
            OPCODE_LD_BU,
            OPCODE_LD_BU_LENGTH,
            emulate_ld_bu as OpcodeHandler,
        ),
    ];
    for &(code, length, handler) in &opcodes {
        if check_op_type(ins, code, length) {
            handler(ins, ctx);
            return;
        }
    }

    panic!("unexpected opcode encountered, ins = {:#x}", ins);
}

/* TLB REFILL HANDLER */
#[no_mangle]
#[naked]
#[link_section = ".tlbrefill_entry"]
extern "C" fn tlb_refill_handler() {
    unsafe {
        asm!(
        "csrwr      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "csrrd      $r12, {LOONGARCH_CSR_PGD}",
        "lddir      $r12, $r12, 3",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "lddir      $r12, $r12, 2",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "lddir      $r12, $r12, 1",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "ldpte      $r12, 0",
        "ldpte      $r12, 1",
        "tlbfill",
        "csrrd      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "ertn",
        LOONGARCH_CSR_TLBRSAVE = const 0x8b,
        LOONGARCH_CSR_PGD = const 0x1b,
        PAGE_WALK_MASK = const 0xfff,
        options(noreturn)
        );
    }
}
