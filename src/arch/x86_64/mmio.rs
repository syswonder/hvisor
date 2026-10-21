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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{
        s2pt::DescriptorAttr,
        vmcs::{Vmcs, VmcsGuest16, VmcsGuestNW},
    },
    cpu_data::{this_cpu_data, this_zone},
    error::{HvError, HvResult},
    memory::{
        addr::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr},
        MMIOAccess, MMIOHandler, PAGE_SIZE,
    },
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{
    mem::size_of,
    ops::Range,
    ptr::{read_volatile, write_volatile},
};
use spin::Mutex;
use x86::controlregs::{Cr0, Cr4};
use x86::irq::PAGE_FAULT_VECTOR;

const MAX_INSTRUCTION_LEN: usize = 15;
const GUEST_PTE_ADDR_MASK: usize = 0x000f_ffff_ffff_f000;

#[derive(Debug)]
enum InstructionError {
    PageFault {
        address: GuestVirtAddr,
        error_code: u32,
    },
    Other(HvError),
}

type InstructionResult<T = ()> = Result<T, InstructionError>;

impl From<HvError> for InstructionError {
    fn from(error: HvError) -> Self {
        Self::Other(error)
    }
}

impl From<x86::vmx::VmFail> for InstructionError {
    fn from(error: x86::vmx::VmFail) -> Self {
        Self::Other(error.into())
    }
}

// Fetch only bytes consumed by the decoder; a valid instruction may end at a page boundary.
struct InstructionStream<F> {
    rip: GuestVirtAddr,
    len: usize,
    fetch: F,
}

impl<F> InstructionStream<F>
where
    F: FnMut(GuestVirtAddr) -> InstructionResult<u8>,
{
    fn new(rip: GuestVirtAddr, fetch: F) -> Self {
        Self { rip, len: 0, fetch }
    }

    fn read_u8(&mut self) -> InstructionResult<u8> {
        if self.len == MAX_INSTRUCTION_LEN {
            return Err(hv_err!(E2BIG, "x86 instruction exceeds 15 bytes").into());
        }
        let address = self
            .rip
            .checked_add(self.len)
            .ok_or_else(|| hv_err!(EFAULT, "instruction address overflow"))?;
        let byte = (self.fetch)(address)?;
        self.len += 1;
        Ok(byte)
    }

    fn read_i8(&mut self) -> InstructionResult<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_i32(&mut self) -> InstructionResult<i32> {
        let bytes = [
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ];
        Ok(i32::from_le_bytes(bytes))
    }
}

struct GuestInstructionBytes {
    // Guest-adjacent pages need not be adjacent in host memory.
    cached_page: Option<(GuestVirtAddr, HostPhysAddr)>,
    real_mode: bool,
}

impl GuestInstructionBytes {
    fn new() -> HvResult<Self> {
        let cr0 = VmcsGuestNW::CR0.read()?;
        Ok(Self {
            cached_page: None,
            real_mode: cr0 & Cr0::CR0_PROTECTED_MODE.bits() == 0,
        })
    }

    fn read(&mut self, address: GuestVirtAddr) -> InstructionResult<u8> {
        let page = address & !(PAGE_SIZE - 1);
        let offset = address & (PAGE_SIZE - 1);
        let host_page = match self.cached_page {
            Some((cached, host_page)) if !self.real_mode && cached == page => host_page,
            _ => {
                let gpa = translate_guest_gva(address, true)?;
                let hpa = gpa_to_hpa(gpa)?;
                // The real-mode CS base may not align with a RIP page boundary.
                if self.real_mode {
                    return Ok(unsafe { read_volatile(hpa as *const u8) });
                }
                let host_page = hpa - offset;
                self.cached_page = Some((page, host_page));
                host_page
            }
        };
        Ok(unsafe { read_volatile((host_page + offset) as *const u8) })
    }
}

pub trait MMIoDevice: Send + Sync {
    fn gpa_range(&self) -> &Vec<Range<usize>>;
    fn read(&self, gpa: GuestPhysAddr) -> HvResult<u64>;
    fn write(&self, gpa: GuestPhysAddr, value: u64, size: usize) -> HvResult;
    fn trigger(&self, signal: usize) -> HvResult;
}

numeric_enum_macro::numeric_enum! {
#[repr(u32)]
#[derive(Debug)]
pub enum RmReg {
    AX = 0,
    CX = 1,
    DX = 2,
    BX = 3,
    SP = 4,
    BP = 5,
    SI = 6,
    DI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    RIP = 16,
    CR0 = 17,
    CR1 = 18,
    CR2  = 19,
    CR3  = 20,
    CR4  = 21,
    GDTR = 22,
    LDTR = 23,
    TR   = 24,
    IDTR = 25,
}
}

impl RmReg {
    fn read(&self) -> HvResult<u64> {
        let gen_regs = this_cpu_data().arch_cpu.regs();
        let res = match self {
            RmReg::AX => gen_regs.rax,
            RmReg::CX => gen_regs.rcx,
            RmReg::DX => gen_regs.rdx,
            RmReg::BX => gen_regs.rbx,
            RmReg::SP => VmcsGuestNW::RSP.read().unwrap() as _,
            RmReg::BP => gen_regs.rbp,
            RmReg::SI => gen_regs.rsi,
            RmReg::DI => gen_regs.rdi,
            RmReg::R8 => gen_regs.r8,
            RmReg::R9 => gen_regs.r9,
            RmReg::R10 => gen_regs.r10,
            RmReg::R11 => gen_regs.r11,
            RmReg::R12 => gen_regs.r12,
            RmReg::R13 => gen_regs.r13,
            RmReg::R14 => gen_regs.r14,
            RmReg::R15 => gen_regs.r15,
            RmReg::RIP => VmcsGuestNW::RIP.read().unwrap() as _,
            RmReg::CR0 => VmcsGuestNW::CR0.read().unwrap() as _,
            RmReg::CR3 => VmcsGuestNW::CR3.read().unwrap() as _,
            RmReg::CR4 => VmcsGuestNW::CR4.read().unwrap() as _,
            RmReg::GDTR => VmcsGuestNW::GDTR_BASE.read().unwrap() as _,
            RmReg::LDTR => VmcsGuestNW::LDTR_BASE.read().unwrap() as _,
            RmReg::TR => VmcsGuestNW::TR_BASE.read().unwrap() as _,
            RmReg::IDTR => VmcsGuestNW::IDTR_BASE.read().unwrap() as _,
            _ => 0,
        };
        Ok(res)
    }

    fn write(&self, new_value: u64, size: usize) -> HvResult {
        let mut gen_regs = this_cpu_data().arch_cpu.regs_mut();

        let mut value = self.read().unwrap();
        value.set_bits(0..(size * 8), new_value.get_bits(0..(size * 8)));

        match self {
            RmReg::AX => gen_regs.rax = value,
            RmReg::CX => gen_regs.rcx = value,
            RmReg::DX => gen_regs.rdx = value,
            RmReg::BX => gen_regs.rbx = value,
            RmReg::SP => VmcsGuestNW::RSP.write(value as _)?,
            RmReg::BP => gen_regs.rbp = value,
            RmReg::SI => gen_regs.rsi = value,
            RmReg::DI => gen_regs.rdi = value,
            RmReg::R8 => gen_regs.r8 = value,
            RmReg::R9 => gen_regs.r9 = value,
            RmReg::R10 => gen_regs.r10 = value,
            RmReg::R11 => gen_regs.r11 = value,
            RmReg::R12 => gen_regs.r12 = value,
            RmReg::R13 => gen_regs.r13 = value,
            RmReg::R14 => gen_regs.r14 = value,
            RmReg::R15 => gen_regs.r15 = value,
            RmReg::RIP => VmcsGuestNW::RIP.write(value as _)?,
            RmReg::CR0 => VmcsGuestNW::CR0.write(value as _)?,
            RmReg::CR3 => VmcsGuestNW::CR3.write(value as _)?,
            RmReg::CR4 => VmcsGuestNW::CR4.write(value as _)?,
            RmReg::GDTR => VmcsGuestNW::GDTR_BASE.write(value as _)?,
            RmReg::LDTR => VmcsGuestNW::LDTR_BASE.write(value as _)?,
            RmReg::TR => VmcsGuestNW::TR_BASE.write(value as _)?,
            RmReg::IDTR => VmcsGuestNW::IDTR_BASE.write(value as _)?,
            _ => {}
        }
        Ok(())
    }
}

/*
G: general registers
E: registers / memory
b: byte
w: word
v: word / dword / qword
*/
numeric_enum_macro::numeric_enum! {
#[repr(u8)]
#[derive(Debug)]
pub enum OneByteOpCode {
    // move r to r/m
    MovEbGb = 0x88,
    MovEvGv = 0x89,
    // move r/m to r
    MovGbEb = 0x8a,
    MovGvEv = 0x8b,
}
}
numeric_enum_macro::numeric_enum! {
#[repr(u8)]
#[derive(Debug)]
pub enum TwoByteOpCode {
    MovZxGvEb = 0xb6,
    MovZxGvEw = 0xb7,
}
}

bitflags::bitflags! {
    #[derive(Debug, PartialEq)]
    struct RexPrefixLow: u8 {
        const BASE = 1 << 0;
        const INDEX = 1 << 1;
        const REGISTERS = 1 << 2;
        const OPERAND_WIDTH = 1 << 3;
    }
}
const REX_PREFIX_HIGH: u8 = 0x4;

const OPERAND_SIZE_OVERRIDE_PREFIX: u8 = 0x66;

const TWO_BYTE_ESCAPE: u8 = 0xf;

enum OprandType {
    Reg(RmReg),
    Gpa(usize),
}

struct ModRM {
    pub _mod: u32,
    pub reg_opcode: u32,
    pub rm: u32,
}

impl ModRM {
    pub fn new(byte: u8, rex: &RexPrefixLow) -> Self {
        let mut reg_opcode = byte.get_bits(3..=5) as u32;
        if rex.contains(RexPrefixLow::REGISTERS) {
            reg_opcode.set_bit(3, true);
        }
        Self {
            _mod: byte.get_bits(6..=7) as _,
            reg_opcode,
            rm: byte.get_bits(0..=2) as _,
        }
    }

    pub fn get_reg(&self) -> InstructionResult<RmReg> {
        self.reg_opcode
            .try_into()
            .map_err(|_| InstructionError::Other(hv_err!(EINVAL, "invalid ModRM register")))
    }

    pub fn get_modrm<F>(&self, inst: &mut InstructionStream<F>) -> InstructionResult<OprandType>
    where
        F: FnMut(GuestVirtAddr) -> InstructionResult<u8>,
    {
        let reg: RmReg = self
            .rm
            .try_into()
            .map_err(|_| hv_err!(EINVAL, "invalid ModRM operand"))?;
        if self._mod == 3 {
            return Ok(OprandType::Reg(reg));
        }
        if self.rm == 4 || (self._mod == 0 && self.rm == 5) {
            return Err(
                hv_err!(ENOSYS, "SIB and RIP-relative MMIO operands are unsupported").into(),
            );
        }
        let reg_val = reg.read()?;
        match self._mod {
            0 => Ok(OprandType::Gpa(gva_to_gpa(reg_val as _)?)),
            1 => {
                let disp = inst.read_i8()? as i64;
                Ok(OprandType::Gpa(gva_to_gpa(
                    reg_val.wrapping_add_signed(disp) as _,
                )?))
            }
            2 => {
                let disp = inst.read_i32()? as i64;
                Ok(OprandType::Gpa(gva_to_gpa(
                    reg_val.wrapping_add_signed(disp) as _,
                )?))
            }
            _ => Err(hv_err!(EINVAL, "invalid ModRM mode").into()),
        }
    }
}

fn gpa_to_hpa(gpa: GuestPhysAddr) -> HvResult<HostPhysAddr> {
    let (hpa, _, _) = unsafe { this_zone().read().gpm().page_table_query(gpa)? };
    Ok(hpa)
}

fn get_page_entry(pt_hpa: HostPhysAddr, pte_id: usize) -> usize {
    unsafe { read_volatile((pt_hpa + pte_id * size_of::<usize>()) as *const usize) }
}

fn gva_to_gpa(gva: GuestVirtAddr) -> HvResult<GuestPhysAddr> {
    translate_guest_gva(gva, false).map_err(|error| match error {
        InstructionError::PageFault { .. } => hv_err!(EFAULT, "guest page is not present"),
        InstructionError::Other(error) => error,
    })
}

fn check_guest_pte(
    entry: usize,
    address: GuestVirtAddr,
    is_instruction: bool,
) -> InstructionResult {
    if entry & 1 == 0 {
        // A missing instruction page must fault in the guest, not the hypervisor.
        let user = (VmcsGuest16::CS_SELECTOR.read()? & 3 != 0) as u32;
        return Err(InstructionError::PageFault {
            address,
            error_code: (user << 2) | ((is_instruction as u32) << 4),
        });
    }
    Ok(())
}

fn translate_guest_gva(
    gva: GuestVirtAddr,
    is_instruction: bool,
) -> InstructionResult<GuestPhysAddr> {
    let cr0 = VmcsGuestNW::CR0.read()?;
    let cr4 = VmcsGuestNW::CR4.read()?;

    // guest hasn't enabled paging, va = pa
    if cr0 & Cr0::CR0_ENABLE_PAGING.bits() == 0 {
        let mut gpa = gva;
        // still in real mode, apply cs
        if cr0 & Cr0::CR0_PROTECTED_MODE.bits() == 0 {
            let cs_selector = VmcsGuest16::CS_SELECTOR.read()? as usize;
            gpa = (cs_selector << 4).wrapping_add(gva);
        }
        return Ok(gpa);
    }

    if cr4 & Cr4::CR4_ENABLE_PAE.bits() == 0 {
        return Err(hv_err!(ENOSYS, "non-PAE guest address translation is unsupported").into());
    }

    // lookup guest page table in long mode

    let p4_gpa = (VmcsGuestNW::CR3.read()?) & !(0xfff);
    let p4_hpa = gpa_to_hpa(p4_gpa)?;
    let p4_entry_id = (gva >> 39) & 0x1ff;
    let p4_entry = get_page_entry(p4_hpa, p4_entry_id);
    check_guest_pte(p4_entry, gva, is_instruction)?;

    let p3_gpa = p4_entry & GUEST_PTE_ADDR_MASK;
    let p3_entry_id = (gva >> 30) & 0x1ff;
    let p3_hpa = gpa_to_hpa(p3_gpa)?;
    let p3_entry = get_page_entry(p3_hpa, p3_entry_id);
    check_guest_pte(p3_entry, gva, is_instruction)?;

    // info!("p3_entry: {:x}", p3_entry);

    if p3_entry & (DescriptorAttr::HUGE_PAGE.bits() as usize) != 0 {
        let page_gpa = p3_entry & GUEST_PTE_ADDR_MASK & !0x3fff_ffff;
        return Ok(page_gpa | (gva & 0x3fffffff));
    }

    let p2_gpa = p3_entry & GUEST_PTE_ADDR_MASK;
    let p2_entry_id = (gva >> 21) & 0x1ff;
    let p2_hpa = gpa_to_hpa(p2_gpa)?;
    let p2_entry = get_page_entry(p2_hpa, p2_entry_id);
    check_guest_pte(p2_entry, gva, is_instruction)?;

    // info!("p2_entry: {:x}", p2_entry);

    if p2_entry & (DescriptorAttr::HUGE_PAGE.bits() as usize) != 0 {
        let page_gpa = p2_entry & GUEST_PTE_ADDR_MASK & !0x1f_ffff;
        return Ok(page_gpa | (gva & 0x1fffff));
    }

    let p1_gpa = p2_entry & GUEST_PTE_ADDR_MASK;
    let p1_entry_id = (gva >> 12) & 0x1ff;
    let p1_hpa = gpa_to_hpa(p1_gpa)?;
    let p1_entry = get_page_entry(p1_hpa, p1_entry_id);
    check_guest_pte(p1_entry, gva, is_instruction)?;

    // info!("p1_entry: {:x}", p1_entry);

    let page_gpa: usize = p1_entry & GUEST_PTE_ADDR_MASK;
    Ok(page_gpa | (gva & 0xfff))
}

fn get_default_operand_size() -> HvResult<usize> {
    let cr0 = VmcsGuestNW::CR0.read()?;
    let mut size = size_of::<u16>();

    // in protection mode
    if cr0 & Cr0::CR0_PROTECTED_MODE.bits() != 0 {
        let gdtr_hpa = gpa_to_hpa(gva_to_gpa(VmcsGuestNW::GDTR_BASE.read()?)?)?;
        let cs_sel = VmcsGuest16::CS_SELECTOR.read()? as usize;
        // info!("gdtr: {:x}", gdtr_hpa);
        let cs_desc = unsafe { *((gdtr_hpa + (cs_sel & !(0x7))) as *const u64) };
        // info!("cs_desc: {:x}", cs_desc);

        // default operation size
        let cs_d = cs_desc.get_bit(54);
        // long mode
        let cs_l = cs_desc.get_bit(53);

        // in 64-bit long mode or set CS.D to 1
        if (!cs_d && cs_l) || cs_d {
            size = size_of::<u32>();
        }
    }

    Ok(size)
}

fn emulate_inst<F>(
    inst: &mut InstructionStream<F>,
    handler: &MMIOHandler,
    mmio: &mut MMIOAccess,
    base: usize,
    arg: usize,
) -> InstructionResult<usize>
where
    F: FnMut(GuestVirtAddr) -> InstructionResult<u8>,
{
    let mmio_offset = |gpa: usize| -> InstructionResult<usize> {
        gpa.checked_sub(base)
            .ok_or_else(|| hv_err!(EFAULT, "MMIO address is below handler region").into())
    };

    let mut size = get_default_operand_size()?;
    let mut size_override = false;
    let mut byte = inst.read_u8()?;

    if byte == OPERAND_SIZE_OVERRIDE_PREFIX {
        if size == size_of::<u32>() {
            size = size_of::<u16>();
        } else {
            size = size_of::<u32>();
        }
        size_override = true;
        byte = inst.read_u8()?;
    }

    let mut rex = RexPrefixLow::from_bits_truncate(0);
    if byte.get_bits(4..=7) == REX_PREFIX_HIGH {
        rex = RexPrefixLow::from_bits_truncate(byte.get_bits(0..=3));
        if rex != RexPrefixLow::REGISTERS {
            return Err(hv_err!(ENOSYS, "unsupported REX prefix in MMIO instruction").into());
        }
        byte = inst.read_u8()?;
    }

    let two_byte = byte == TWO_BYTE_ESCAPE;
    let opcode_byte = if two_byte { inst.read_u8()? } else { byte };

    if !two_byte {
        let opcode: OneByteOpCode = opcode_byte
            .try_into()
            .map_err(|_| hv_err!(ENOSYS, "unsupported MMIO instruction opcode"))?;

        if !size_override {
            size = match opcode {
                OneByteOpCode::MovEbGb | OneByteOpCode::MovGbEb => size_of::<u8>(),
                _ => size,
            };
        }

        match opcode {
            OneByteOpCode::MovEbGb | OneByteOpCode::MovEvGv => {
                let mod_rm = ModRM::new(inst.read_u8()?, &rex);
                let src_val = mod_rm.get_reg()?.read()?;
                let dst = mod_rm.get_modrm(inst)?;
                match dst {
                    OprandType::Reg(reg) => {
                        reg.write(src_val, size)?;
                    }
                    OprandType::Gpa(gpa) => {
                        mmio.address = mmio_offset(gpa)?;
                        mmio.is_write = true;
                        mmio.size = size;
                        mmio.value = src_val as _;
                        handler(mmio, arg)?;
                    }
                }
                Ok(inst.len)
            }
            OneByteOpCode::MovGbEb | OneByteOpCode::MovGvEv => {
                let mod_rm = ModRM::new(inst.read_u8()?, &rex);
                let dst = mod_rm.get_reg()?;
                let src = mod_rm.get_modrm(inst)?;
                let src_val = match src {
                    OprandType::Reg(reg) => reg.read()?,
                    OprandType::Gpa(gpa) => {
                        mmio.address = mmio_offset(gpa)?;
                        mmio.is_write = false;
                        mmio.size = size;
                        mmio.value = 0;
                        handler(mmio, arg)?;
                        mmio.value as u64
                    }
                };
                dst.write(src_val, size)?;
                Ok(inst.len)
            }
            _ => Err(hv_err!(
                ENOSYS,
                format!("Unimplemented opcode: 0x{:x}", opcode as u8)
            )
            .into()),
        }
    } else {
        let opcode: TwoByteOpCode = opcode_byte
            .try_into()
            .map_err(|_| hv_err!(ENOSYS, "unsupported two-byte MMIO opcode"))?;

        if !size_override {
            size = match opcode {
                TwoByteOpCode::MovZxGvEb => size_of::<u8>(),
                TwoByteOpCode::MovZxGvEw => size_of::<u16>(),
                _ => size,
            };
        }

        match opcode {
            TwoByteOpCode::MovZxGvEb | TwoByteOpCode::MovZxGvEw => {
                let mod_rm = ModRM::new(inst.read_u8()?, &rex);
                let dst = mod_rm.get_reg()?;
                let src = mod_rm.get_modrm(inst)?;
                let src_val = match src {
                    OprandType::Reg(reg) => reg.read()?,
                    OprandType::Gpa(gpa) => {
                        mmio.address = mmio_offset(gpa)?;
                        mmio.is_write = false;
                        mmio.size = size;
                        mmio.value = 0;
                        handler(mmio, arg)?;
                        mmio.value as u64
                    }
                };
                let src_val_zero_extend = match size {
                    1 => src_val.get_bits(0..8),
                    2 => src_val.get_bits(0..16),
                    4 => src_val.get_bits(0..32),
                    _ => src_val,
                };

                dst.write(src_val_zero_extend, 8)?;
                Ok(inst.len)
            }
            _ => Err(hv_err!(
                ENOSYS,
                format!("Unimplemented opcode: 0x{:x}", opcode as u8)
            )
            .into()),
        }
    }
}

pub fn instruction_emulator(
    handler: &MMIOHandler,
    mmio: &mut MMIOAccess,
    base: usize,
    arg: usize,
) -> HvResult {
    let rip = VmcsGuestNW::RIP.read()?;
    let mut bytes = GuestInstructionBytes::new()?;
    let mut inst = InstructionStream::new(rip, |address| bytes.read(address));
    match emulate_inst(&mut inst, handler, mmio, base, arg) {
        Ok(len) => this_cpu_data().arch_cpu.advance_guest_rip(len as u8),
        Err(InstructionError::PageFault {
            address,
            error_code,
        }) => {
            // Retry the same instruction after the guest handles the page fault.
            this_cpu_data().arch_cpu.set_guest_cr2(address);
            Vmcs::inject_interrupt(PAGE_FAULT_VECTOR, Some(error_code))
        }
        Err(InstructionError::Other(error)) => Err(error),
    }
}

pub fn mmio_empty_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    if !mmio.is_write {
        mmio.value = 0;
    }
    Ok(())
}
