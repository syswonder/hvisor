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
use super::cpu::ArchCpu;
use crate::arch::csr::read_csr;
use crate::arch::csr::*;
use crate::arch::sbi::sbi_vs_handler;
#[cfg(feature = "aia")]
use crate::device::irqchip::aia::aplic::{host_aplic, vaplic_emul_handler};
#[cfg(feature = "plic")]
use crate::device::irqchip::plic::*;
use crate::event::check_events;
use crate::memory::{mmio_handle_access, MMIOAccess};
use crate::memory::{GuestPhysAddr, HostPhysAddr};
use crate::percpu::this_cpu_data;
use crate::platform::__board::*;
use core::arch::{asm, global_asm};
use riscv::register::stvec::TrapMode;
use riscv::register::{sie, stvec};
use riscv_decode::Instruction;
use riscv_h::register::hvip;

extern "C" {
    fn _hyp_trap_vector();
}

global_asm!(include_str!("trap.S"),
sync_exception_handler=sym sync_exception_handler,
interrupts_arch_handle=sym interrupts_arch_handle);

#[allow(non_snake_case)]
pub mod ExceptionType {
    pub const ECALL_VU: usize = 8;
    pub const ECALL_VS: usize = 10;
    pub const LOAD_GUEST_PAGE_FAULT: usize = 21;
    pub const STORE_GUEST_PAGE_FAULT: usize = 23;
}

pub const CAUSE_STRINGS: [&str; 24] = [
    "Instruction address misaligned",
    "Instruction access fault",
    "Illegal instruction",
    "Breakpoint",
    "Load address misaligned",
    "Load access fault",
    "Store/AMO address misaligned",
    "Store/AMO access fault",
    "Environment call from U-mode or VU-mode",
    "Environment call from S-mode",
    "Environment call from VS-mode",
    "Environment call from M-mode",
    "Instruction page fault",
    "Load page fault",
    "Reserved (14)",
    "Store/AMO page fault",
    "Reserved (16)",
    "Reserved (17)",
    "Reserved (18)",
    "Reserved (19)",
    "Instruction guest-page fault",
    "Load guest-page fault",
    "Virtual instruction",
    "Store/AMO guest-page fault",
];

#[allow(non_snake_case)]
pub mod InterruptType {
    pub const SSI: usize = 1;
    pub const STI: usize = 5;
    pub const SEI: usize = 9;
}

// standard ins[14:12]
pub const INS_FUNCT3_MASK: usize = 0x00007000;
// standard ins[6:0]
pub const INS_OPCODE_MASK: usize = 0x0000007f;
pub const OPCODE_LOAD: usize = 0x03;
pub const OPCODE_STORE: usize = 0x23;

// compressed ins[15:13]
pub const INS_C_FUNCT3_MASK: usize = 0xe000;
// compressed ins[1:0]
pub const INS_C_OPCODE_MASK: usize = 0x0003;
pub const INS_C_LW: usize = 0x4000;
pub const INS_C_SW: usize = 0xc000;

pub const INS_RS1_MASK: usize = 0x000f8000;
pub const INS_RS2_MASK: usize = 0x01f00000;
pub const INS_RD_MASK: usize = 0x00000f80;

/// Set the trap vector.
pub fn install_trap_vector() {
    use riscv::register::stvec::Stvec;
    let mut stvec = Stvec::from_bits(0);
    stvec.set_address(_hyp_trap_vector as usize);
    stvec.set_trap_mode(TrapMode::Direct);
    unsafe {
        stvec::write(stvec);
    }
}

/// Handle synchronous exceptions.
pub fn sync_exception_handler(current_cpu: &mut ArchCpu) {
    trace!("current_cpu: stack{:#x}", current_cpu.stack_top);
    let trap_code = riscv::register::scause::read().code();
    trace!("CSR_SCAUSE: {}", trap_code);

    if !riscv_h::register::hstatus::read().spv() {
        // Hvisor don't handle sync exception which occurs in hvisor self (HS-mode).
        // If sync exception occurs, hvisor will panic!
        panic!("exception from HS mode");
    }

    let trap_value = riscv_h::register::htval::read();
    let trap_ins = riscv_h::register::htinst::read();
    let trap_pc = riscv::register::sepc::read();
    trace!("CSR_HTVAL: {:#x}", trap_value);
    trace!("CSR_HTINST: {:#x}", trap_ins);
    trace!("CSR_SEPC: {:#x}", trap_pc);

    match trap_code {
        ExceptionType::ECALL_VS => {
            trace!("ECALL_VS");
            sbi_vs_handler(current_cpu);
            current_cpu.sepc += 4;          // For ecall, skip the ecall instruction.
        }
        ExceptionType::LOAD_GUEST_PAGE_FAULT => {
            trace!("LOAD_GUEST_PAGE_FAULT");
            guest_page_fault_handler(current_cpu);
        }
        ExceptionType::STORE_GUEST_PAGE_FAULT => {
            trace!("STORE_GUEST_PAGE_FAULT");
            guest_page_fault_handler(current_cpu);
        }
        _ => {
            let raw_inst = read_inst(trap_pc);
            let inst = riscv_decode::decode(raw_inst);
            warn!("CPU {} sync exception, sepc: {:#x}", current_cpu.cpuid, current_cpu.sepc);
            warn!("Trap cause code: {}", trap_code);
            warn!("htval: {:#x}, htinst: {:#x}", trap_value, trap_ins);
            warn!("trap instruction: {:?}", inst);
            panic!("Unhandled sync exception");
        }
    }
}

/// Check if the instruction is compressed.
#[inline(always)]
pub fn ins_is_compressed(ins: usize) -> bool {
    // this is used for standard instruction.
    (ins & 0x3) != 3
}

/// Check if the instruction in htinst is a pseudo instruction or not.
#[inline(always)]
pub fn ins_is_preudo(ins: usize) -> bool {
    /*
     * riscv priv spec:
     * for htinst, all preudo instruction's[1:0] are 0.
     */
    (ins & 0x3) == 0
}

/// Decode instruction to mmio access information.
pub fn ins_ldst_decode(ins: usize) -> (usize, bool, bool) {
    /*
     * For htinst
     * Standard compressed instruction will expand to 32-bit equivalent instruction.
     * Due to we don't read instruction from guest memory,
     * So it will never be compressed instruction here.
     */

    if ins_is_compressed(ins) {
        panic!("trap instruction is compressed.");
        // if (ins & (INS_C_OPCODE_MASK|INS_C_FUNCT3_MASK) != INS_C_LW) && (ins & (INS_C_OPCODE_MASK|INS_C_FUNCT3_MASK) != INS_C_SW) {
        //     panic!("trap instruction is not C.LW or C.SW, inst 0x{:04x}.", ins);
        // }
        // let size: usize = 4;
        // let is_write = ins & (INS_C_OPCODE_MASK|INS_C_FUNCT3_MASK) == INS_C_SW;
        // (size, is_write, true)
    } else {
        if (ins & INS_OPCODE_MASK != OPCODE_LOAD) && (ins & INS_OPCODE_MASK != OPCODE_STORE) {
            panic!("trap instruction is not Load or Store, inst 0x{:08x}.", ins);
        }
        let funct3: usize = (ins & INS_FUNCT3_MASK) >> 12;
        let size = match funct3 {
            0b000 => 1, // LB/LBU
            0b001 => 2, // LH/LHU
            0b010 => 4, // LW
            0b011 => 8, // LD
            0b100 => 1, // SB
            0b101 => 2, // SH
            0b110 => 4, // SW
            0b111 => 8, // SD
            _ => {
                panic!("trap instruction's funct3 is INVALID, inst 0x{:08x}.", ins);
            }
        };
        let is_write = ins & INS_OPCODE_MASK == OPCODE_STORE;
        let sign_ext = (funct3 & 0x4) == 0;
        (size, is_write, sign_ext)
    }
}

/// Handle guest page fault sync exception.
pub fn guest_page_fault_handler(current_cpu: &mut ArchCpu) {
    #[cfg(feature = "plic")]
    {
        use riscv_h::register::{htinst, htval, stval};
        // htval: Hypervisor bad guest physical address.
        let addr: usize = (htval::read() << 2) | (stval::read() & 0x3);
        // htinst: Hypervisor trap instruction (transformed).
        let mut trap_ins = htinst::read();

        /*
         * For a standard compressed instruction (16-bit size), the transformed instruction is found as follows:
         * 1. Expand the compressed instruction to its 32-bit equivalent.
         * 2. Transform the 32-bit equivalent instruction.
         * 3. Replace bit 1 with a 0.
         *
         * Bits[1:0] of a transformed standard instruction will be binary 01
         * if the trapping instruction is compressed and 11 if not.
         */
        let ins_size = match trap_ins & 0x3 {
            0x1 => 2,
            0x3 => 4,
            _ => panic!("Invalid instruction size."),
        };
        current_cpu.sepc += ins_size;

        /* Determine trapped instruction */
        if trap_ins & 0x1 == 0x1 {
            /*
             * Bit[0] == 1, and replacing bit 1 with 1 makes the value
             * into a valid encoding of a standard instruction.
             */
            trap_ins = trap_ins | 0x2;
        } else if ins_is_preudo(trap_ins) {
            /*
             * htinst's value is one of the special pseudoinstructions
             * all of which have bits 1:0 equal to 00.
             */
            panic!("No support for htinst pseudo instruction.");
        } else {
            /*
             * Bit[0] == 0 implies trapped instruction value is
             * zero or special value.
             */
            panic!("No support for reading instruction from guest memory.");
        }

        // decode instruction to get size, is_write and sign_ext.
        let (size, is_write, sign_ext) = ins_ldst_decode(trap_ins);
        let ins_rd = (INS_RD_MASK & trap_ins) >> 7;
        let ins_rs2 = (INS_RS2_MASK & trap_ins) >> 20;

        // warn!("guest page fault at {:#x}, trap_ins: {:08x}, size: {}, is_write: {}, sign_ext: {}", addr, trap_ins, size, is_write, sign_ext);

        // create mmio access struct.
        let mut mmio_access = MMIOAccess {
            address: addr as _,
            size: size as _,
            is_write: is_write as _,
            value: if is_write {
                // for store instruction, x[rs2] will be written into memory.
                if ins_rs2 == 0 {
                    0
                } else {
                    current_cpu.x[ins_rs2] as _
                }
            } else {
                // for load instruction, value is used to store the result.
                0
            },
        };

        match mmio_handle_access(&mut mmio_access) {
            Ok(_) => {
                if !is_write && ins_rd != 0 {
                    // for load instruction, x[rd] will be written.
                    if sign_ext {
                        // note: this is used for 64bit system.
                        mmio_access.value = (((mmio_access.value << (64 - 8 * size)) as i64)
                            >> (64 - 8 * size))
                            as usize;
                    }
                    current_cpu.x[ins_rd] = mmio_access.value as _;
                }
            }
            Err(e) => {
                panic!("mmio_handle_access: {:#x?}", e);
            }
        }
    }
    #[cfg(feature = "aia")]
    {
        let addr: HostPhysAddr = read_csr!(CSR_HTVAL) << 2;
        trace!("guest page fault at {:#x}", addr);

        let host_aplic_base = host_aplic().read().base;
        let host_aplic_size = host_aplic().read().size;

        if addr >= host_aplic_base && addr < host_aplic_base + host_aplic_size {
            trace!("APLIC access");
            let mut inst: u32 = read_csr!(CSR_HTINST) as u32;
            let mut ins_size: usize = 0;
            if inst == 0 {
                let inst_addr: GuestPhysAddr = current_cpu.sepc;
                inst = read_inst(inst_addr);
                ins_size = if inst & 0x3 == 3 { 4 } else { 2 };
            } else if inst == 0x3020 || inst == 0x3000 {
                error!("fault on 1st stage page table walk");
            } else {
                ins_size = if (inst) & 0x2 == 0 { 2 } else { 4 };
                inst = inst | 0b10;
                // error!("unhandled guest page fault at {:#x}", addr);
            }
            // let (len, inst) = decode_inst(inst);
            let (_, inst) = decode_inst(inst);

            if let Some(inst) = inst {
                vaplic_emul_handler(current_cpu, addr, inst);
                current_cpu.sepc += ins_size;
            } else {
                error!("Invalid instruction at {:#x}", current_cpu.sepc);
            }
        } else {
            panic!("CPU {} unmaped memmory at {:#x}", current_cpu.cpuid, addr);
        }
    }
}

/// Read instruction from guest memory.
fn read_inst(addr: GuestPhysAddr) -> u32 {
    let mut ins: u32;
    if addr & 0b1 != 0 {
        error!("trying to read guest unaligned instruction");
    }
    //
    //  Read 16 bits at a time to make sure the access is aligned. If the instruction is not
    //  compressed, read the following 16-bits.
    //
    ins = hlvxhu(addr) as u32;
    if (ins & 0b11) == 3 {
        ins |= (hlvxhu(addr + 2) as u32) << 16;
    }
    ins
}

/// Hypervisor Virtual-Machine Load and Store Instruction.
/// HLVX.HU emulate VS load instruction.
fn hlvxhu(addr: GuestPhysAddr) -> u64 {
    let mut value: u64;
    unsafe {
        asm!(
            ".insn r 0x73, 0x4, 0x32, {0}, {1}, x3",
            out(reg) value,
            in(reg) addr,
        );
    }
    value
}

/// Decode risc-v instruction, return (inst len, inst).
fn decode_inst(inst: u32) -> (usize, Option<Instruction>) {
    let i1 = inst as u16;
    let len = riscv_decode::instruction_length(i1);
    let inst = match len {
        2 => i1 as u32,
        4 => inst as u32,
        _ => unreachable!(),
    };
    (len, riscv_decode::decode(inst).ok())
}

/// Handle interrupts which hvisor receives.
pub fn interrupts_arch_handle(current_cpu: &mut ArchCpu) {
    trace!("interrupts_arch_handle @CPU{}", current_cpu.cpuid);
    let trap_code = unsafe { riscv::register::scause::read().code() };
    match trap_code {
        InterruptType::STI => {
            // Inject timer interrupt to VS.
            handle_timer_interrupt(current_cpu);
        },
        InterruptType::SSI => {
            // Get event to handle and clear software interrupt pending bit.
            handle_software_interrupt(current_cpu);
        }
        InterruptType::SEI => {
            // Write external interrupt to vplic and then inject to VS.
            handle_external_interrupt(current_cpu);
        }
        _ => {
            panic!(
                "unhandled trap {:#x},sepc: {:#x}",
                trap_code, current_cpu.sepc
            );
        }
    }
}

/// Handle supervisor timer interrupt.
pub fn handle_timer_interrupt(current_cpu: &mut ArchCpu) {
    unsafe {
        hvip::set_vstip();
        sie::clear_stimer();
    }
}

/// Handle supervisor software interrupt.
pub fn handle_software_interrupt(current_cpu: &mut ArchCpu) {
    while check_events() { 
        // Get next event to handle, it is handled in check_events function.
    }
    unsafe { riscv::register::sip::clear_ssoft(); }
}

/// Handle supervisor external interrupt.
pub fn handle_external_interrupt(current_cpu: &mut ArchCpu) {
    #[cfg(feature = "plic")]
    {
        // Note: in hvisor, all external interrupts are assigned to VS.
        // 1. claim hw irq.
        let context_id = 2 * this_cpu_data().id + 1;
        let irq_id = host_plic().claim(context_id);

        // If this irq has been claimed, it will be 0.
        if irq_id == 0 { return; }

        // 2. inject hw irq to zone.
        inject_irq(irq_id as usize, true);
    }
    #[cfg(feature = "aia")]
    {
        panic!("HS extensional interrupt")
    }
}


