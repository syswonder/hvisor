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
        vmcs::{VmcsGuest16, VmcsGuestNW},
    },
    cpu_data::{this_cpu_data, this_zone},
    error::HvResult,
    memory::{
        addr::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr},
        MMIOAccess, MMIOHandler,
    },
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{mem::size_of, ops::Range, ptr::write_volatile, slice::from_raw_parts};
use spin::Mutex;
use x86::controlregs::{Cr0, Cr4};

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

/// Legacy prefixes that may precede the opcode of a memory-accessing
/// instruction. Only the operand-size prefix affects the operand width of the
/// MOV forms decoded here; the rest are accepted and skipped so that a prefixed
/// access is decoded rather than faulting the hypervisor.
const LEGACY_PREFIXES: [u8; 10] = [
    0x67, // address-size override
    0xf0, // LOCK
    0xf2, // REPNE
    0xf3, // REP
    0x2e, // CS segment override
    0x36, // SS segment override
    0x3e, // DS segment override
    0x26, // ES segment override
    0x64, // FS segment override
    0x65, // GS segment override
];

// `len` is the number of trailing bytes (SIB + displacement) the operand adds to
// the instruction length.
enum OprandType {
    Reg { reg: RmReg, len: usize },
    Mem { len: usize },
}

struct ModRM {
    pub _mod: u32,
    pub reg_opcode: u32,
    pub rm: u32,
    pub rm_ext: u32,
}

impl ModRM {
    pub fn new(byte: u8, rex: &RexPrefixLow) -> Self {
        let mut reg_opcode = byte.get_bits(3..=5) as u32;
        if rex.contains(RexPrefixLow::REGISTERS) {
            reg_opcode.set_bit(3, true);
        }
        let rm = byte.get_bits(0..=2) as u32;
        let mut rm_ext = rm;
        if rex.contains(RexPrefixLow::BASE) {
            rm_ext.set_bit(3, true);
        }
        Self {
            _mod: byte.get_bits(6..=7) as _,
            reg_opcode,
            rm,
            rm_ext,
        }
    }

    pub fn get_reg(&self) -> RmReg {
        self.reg_opcode.try_into().unwrap()
    }

    /// Decode the r/m operand. For a memory operand the faulting guest-physical
    /// address is taken from the VMX exit information rather than recomputed
    /// here, so only the encoded length (ModRM + optional SIB + displacement)
    /// is returned; this keeps decoding correct for SIB and RIP-relative forms
    /// without re-deriving the effective address.
    pub fn get_modrm(&self, inst: &[u8], sib_id: usize) -> Option<OprandType> {
        if self._mod == 3 {
            return Some(OprandType::Reg {
                reg: self.rm_ext.try_into().ok()?,
                len: 0,
            });
        }

        let has_sib = self.rm == 4;
        let mut len = if has_sib { 1 } else { 0 };
        match self._mod {
            0 => {
                if self.rm == 5 {
                    len += 4; // RIP-relative disp32
                } else if has_sib && (*inst.get(sib_id)? & 0x7) == 5 {
                    len += 4; // SIB with no base -> disp32
                }
            }
            1 => len += 1, // disp8
            2 => len += 4, // disp32
            _ => {}
        }
        Some(OprandType::Mem { len })
    }
}

fn gpa_to_hpa(gpa: GuestPhysAddr) -> HvResult<HostPhysAddr> {
    let (hpa, _, _) = unsafe { this_zone().read().gpm().page_table_query(gpa)? };
    Ok(hpa)
}

fn get_page_entry(pt_hpa: HostPhysAddr, pte_id: usize) -> usize {
    unsafe { (*((pt_hpa + (pte_id * size_of::<usize>())) as *const usize)) & 0x7ffffffffffffusize }
}

fn gva_to_gpa(gva: GuestVirtAddr) -> HvResult<GuestPhysAddr> {
    let mut gpa: GuestPhysAddr = 0;
    let cr0 = VmcsGuestNW::CR0.read()?;
    let cr4 = VmcsGuestNW::CR4.read()?;

    // guest hasn't enabled paging, va = pa
    if cr0 & Cr0::CR0_ENABLE_PAGING.bits() == 0 {
        gpa = gva;
        // still in real mode, apply cs
        if cr0 & Cr0::CR0_PROTECTED_MODE.bits() == 0 {
            let cs_selector = VmcsGuest16::CS_SELECTOR.read()? as usize;
            gpa = (cs_selector << 4) | gva;
        }
        return Ok(gpa);
    }

    if cr4 & Cr4::CR4_ENABLE_PAE.bits() == 0 {
        panic!("protected mode gva_to_gpa not implemented yet!");
    }

    // lookup guest page table in long mode
    let mut p4_gpa = (VmcsGuestNW::CR3.read()?) & !(0xfff);

    // With 5-level paging (CR4.LA57) CR3 points at the PML5 table; resolve the
    // extra level down to the PML4 before the standard four-level walk.
    const CR4_LA57: usize = 1 << 12;
    if cr4 & CR4_LA57 != 0 {
        let p5_hpa = gpa_to_hpa(p4_gpa)?;
        let p5_entry = get_page_entry(p5_hpa, (gva >> 48) & 0x1ff);
        p4_gpa = p5_entry & !(0xfff);
    }

    let p4_hpa = gpa_to_hpa(p4_gpa)?;
    let p4_entry_id = (gva >> 39) & 0x1ff;
    let p4_entry = get_page_entry(p4_hpa, p4_entry_id);

    let p3_gpa = p4_entry & !(0xfff);
    let p3_entry_id = (gva >> 30) & 0x1ff;
    let p3_hpa = gpa_to_hpa(p3_gpa)?;
    let p3_entry = get_page_entry(p3_hpa, p3_entry_id);

    // info!("p3_entry: {:x}", p3_entry);

    if p3_entry & (DescriptorAttr::HUGE_PAGE.bits() as usize) != 0 {
        let page_gpa = p3_entry & !(0xfff);
        return Ok(page_gpa | (gva & 0x3fffffff));
    }

    let p2_gpa = p3_entry & !(0xfff);
    let p2_entry_id = (gva >> 21) & 0x1ff;
    let p2_hpa = gpa_to_hpa(p2_gpa)?;
    let p2_entry = get_page_entry(p2_hpa, p2_entry_id);

    // info!("p2_entry: {:x}", p2_entry);

    if p2_entry & (DescriptorAttr::HUGE_PAGE.bits() as usize) != 0 {
        let page_gpa = p2_entry & !(0xfff);
        return Ok(page_gpa | (gva & 0x1fffff));
    }

    let p1_gpa = p2_entry & !(0xfff);
    let p1_entry_id = (gva >> 12) & 0x1ff;
    let p1_hpa = gpa_to_hpa(p1_gpa)?;
    let p1_entry = get_page_entry(p1_hpa, p1_entry_id);

    // info!("p1_entry: {:x}", p1_entry);

    let page_gpa: usize = p1_entry & !(0xfff);
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

fn emulate_inst(
    inst: &[u8],
    handler: &MMIOHandler,
    mmio: &mut MMIOAccess,
    base: usize,
) -> HvResult<usize> {
    // Every index into `inst` below is bounds-checked: the bytes come from guest
    // memory and must never be able to panic the hypervisor.
    let next = |i: usize| -> HvResult<u8> {
        inst.get(i)
            .copied()
            .ok_or_else(|| hv_err!(EINVAL, "truncated instruction stream"))
    };
    let modrm_err = || hv_err!(ENOSYS, "unsupported MMIO ModRM operand");

    let mut size = get_default_operand_size()?;
    let mut size_override = false;
    let mut cur_id = 0;

    // Consume legacy prefixes; the operand-size prefix selects 16-bit v-form
    // operands (idempotent, not a toggle), the rest are accepted and ignored.
    loop {
        let byte = next(cur_id)?;
        if byte == OPERAND_SIZE_OVERRIDE_PREFIX {
            size_override = true;
        } else if !LEGACY_PREFIXES.contains(&byte) {
            break;
        }
        cur_id += 1;
    }
    // The operand-size prefix flips the default v-form width once (32<->16).
    if size_override {
        size = if size == size_of::<u32>() {
            size_of::<u16>()
        } else {
            size_of::<u32>()
        };
    }

    let mut rex = RexPrefixLow::from_bits_truncate(0);
    if next(cur_id)?.get_bits(4..=7) == REX_PREFIX_HIGH {
        rex = RexPrefixLow::from_bits_truncate(next(cur_id)?.get_bits(0..=3));
        cur_id += 1;
    }
    // REX.W selects a 64-bit operand and takes precedence over the 0x66 prefix.
    let rex_w = rex.contains(RexPrefixLow::OPERAND_WIDTH);

    let mut two_byte = false;
    if next(cur_id)? == TWO_BYTE_ESCAPE {
        two_byte = true;
        cur_id += 1;
    }

    if !two_byte {
        let opcode = OneByteOpCode::try_from(next(cur_id)?).map_err(|_| {
            hv_err!(
                ENOSYS,
                format!("unsupported MMIO opcode {:#x}", inst[cur_id])
            )
        })?;
        cur_id += 1;

        // Byte forms are always one byte regardless of 0x66/REX.W; REX.W promotes
        // the v-form operand to 64-bit, otherwise the operand-size prefix applies.
        let byte_form = matches!(opcode, OneByteOpCode::MovEbGb | OneByteOpCode::MovGbEb);
        if byte_form {
            size = size_of::<u8>();
        } else if rex_w {
            size = size_of::<u64>();
        }

        match opcode {
            OneByteOpCode::MovEbGb | OneByteOpCode::MovEvGv => {
                let mod_rm = ModRM::new(next(cur_id)?, &rex);
                cur_id += 1;

                let src_val = mod_rm.get_reg().read()?;

                match mod_rm.get_modrm(inst, cur_id).ok_or_else(modrm_err)? {
                    OprandType::Reg { reg, len } => {
                        cur_id += len;
                        reg.write(src_val, size)?;
                    }
                    OprandType::Mem { len } => {
                        cur_id += len;

                        mmio.is_write = true;
                        mmio.size = size;
                        mmio.value = src_val as _;

                        handler(mmio, base)?;
                    }
                }

                Ok(cur_id)
            }
            OneByteOpCode::MovGbEb | OneByteOpCode::MovGvEv => {
                let mod_rm = ModRM::new(next(cur_id)?, &rex);
                cur_id += 1;

                let dst = mod_rm.get_reg();

                let src_val = match mod_rm.get_modrm(inst, cur_id).ok_or_else(modrm_err)? {
                    OprandType::Reg { reg, len } => {
                        cur_id += len;
                        reg.read()?
                    }
                    OprandType::Mem { len } => {
                        cur_id += len;

                        mmio.is_write = false;
                        mmio.size = size;
                        mmio.value = 0;

                        handler(mmio, base)?;
                        mmio.value as u64
                    }
                };

                dst.write(src_val, size)?;
                Ok(cur_id)
            }
        }
    } else {
        let opcode = TwoByteOpCode::try_from(next(cur_id)?).map_err(|_| {
            hv_err!(
                ENOSYS,
                format!("unsupported MMIO opcode 0f {:#x}", inst[cur_id])
            )
        })?;
        cur_id += 1;

        // The memory (source) operand width of MOVZX is fixed by the opcode and
        // is unaffected by the 0x66 / REX.W prefixes (which only widen the
        // destination, always written zero-extended below).
        size = match opcode {
            TwoByteOpCode::MovZxGvEb => size_of::<u8>(),
            TwoByteOpCode::MovZxGvEw => size_of::<u16>(),
        };

        match opcode {
            TwoByteOpCode::MovZxGvEb | TwoByteOpCode::MovZxGvEw => {
                let mod_rm = ModRM::new(next(cur_id)?, &rex);
                cur_id += 1;

                let dst = mod_rm.get_reg();

                let src_val = match mod_rm.get_modrm(inst, cur_id).ok_or_else(modrm_err)? {
                    OprandType::Reg { reg, len } => {
                        cur_id += len;
                        reg.read()?
                    }
                    OprandType::Mem { len } => {
                        cur_id += len;

                        mmio.is_write = false;
                        mmio.size = size;
                        mmio.value = 0;

                        handler(mmio, base)?;
                        mmio.value as u64
                    }
                };
                let src_val_zero_extend = match size {
                    1 => src_val.get_bits(0..8),
                    2 => src_val.get_bits(0..16),
                    _ => src_val,
                };

                dst.write(src_val_zero_extend, 8)?;
                Ok(cur_id)
            }
        }
    }
}

pub fn instruction_emulator(handler: &MMIOHandler, mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let rip_hpa = gpa_to_hpa(gva_to_gpa(VmcsGuestNW::RIP.read()?)?)? as *const u8;
    let inst = unsafe { from_raw_parts(rip_hpa, 15) }.to_vec();

    let len = emulate_inst(&inst, handler, mmio, base)?;
    this_cpu_data().arch_cpu.advance_guest_rip(len as _)?;

    Ok(())
}

pub fn mmio_empty_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    if !mmio.is_write {
        mmio.value = 0;
    }
    Ok(())
}
