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
        vmcs::{VmcsGuest32, VmcsGuest64, VmcsGuestNW},
    },
    cpu_data::{this_cpu_data, this_zone},
    error::HvResult,
    memory::{
        addr::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr},
        MMIOAccess, MMIOHandler,
    },
};
use alloc::vec::Vec;
use bit_field::BitField;
use core::{mem::size_of, ops::Range, slice::from_raw_parts};
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
            RmReg::SP => VmcsGuestNW::RSP.read()? as _,
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
            RmReg::RIP => VmcsGuestNW::RIP.read()? as _,
            RmReg::CR0 => VmcsGuestNW::CR0.read()? as _,
            RmReg::CR3 => VmcsGuestNW::CR3.read()? as _,
            RmReg::CR4 => VmcsGuestNW::CR4.read()? as _,
            RmReg::GDTR => VmcsGuestNW::GDTR_BASE.read()? as _,
            RmReg::LDTR => VmcsGuestNW::LDTR_BASE.read()? as _,
            RmReg::TR => VmcsGuestNW::TR_BASE.read()? as _,
            RmReg::IDTR => VmcsGuestNW::IDTR_BASE.read()? as _,
            _ => 0,
        };
        Ok(res)
    }

    fn write(&self, new_value: u64, size: usize) -> HvResult {
        let mut gen_regs = this_cpu_data().arch_cpu.regs_mut();

        // x86-64 register-write width semantics: a 32-bit write zero-extends into
        // the full 64-bit register, a 64-bit write replaces all bits, and 8/16-bit
        // writes leave the upper bits unchanged.
        let value = match size {
            8 => new_value,
            4 => new_value as u32 as u64,
            _ => {
                let mut v = self.read()?;
                v.set_bits(0..(size * 8), new_value.get_bits(0..(size * 8)));
                v
            }
        };

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

// Checked reads into the fetched instruction bytes. The fetch stops at an
// unmapped continuation page, so the buffer may be shorter than a full
// instruction; a decoder that needs a byte that was not fetched returns an error
// (the access is unemulatable and is turned into a guest #UD) rather than panic.
fn inst_byte(inst: &[u8], i: usize) -> HvResult<u8> {
    inst.get(i)
        .copied()
        .ok_or_else(|| hv_err!(ENOSYS, "truncated mmio instruction"))
}

fn inst_slice(inst: &[u8], range: core::ops::Range<usize>) -> HvResult<&[u8]> {
    inst.get(range)
        .ok_or_else(|| hv_err!(ENOSYS, "truncated mmio instruction"))
}

// A decoded memory operand: its guest-physical address and the number of
// displacement/SIB bytes consumed after the ModRM byte. A register-direct operand
// (mod=3) is not a memory access and is rejected during decode, so the decoder
// only ever yields a memory operand.
struct MemOperand {
    gpa: usize,
    len: usize,
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

    pub fn get_reg(&self) -> RmReg {
        self.reg_opcode.try_into().unwrap()
    }

    pub fn get_modrm(
        &self,
        inst: &[u8],
        disp_id: usize,
        rex: &RexPrefixLow,
    ) -> HvResult<MemOperand> {
        let as_reg =
            |n: u32| -> HvResult<RmReg> { RmReg::try_from(n).map_err(|_| hv_err!(EFAULT)) };
        // Check if we need to use SIB - when rm field is 4 (and not register mode)
        if self.rm == 4 && self._mod != 3 {
            // Read SIB byte
            let sib_byte = inst_byte(inst, disp_id)?;
            let scale = sib_byte.get_bits(6..8) as u32;
            let raw_base = sib_byte.get_bits(0..3) as u32;
            let mut index = sib_byte.get_bits(3..6) as u32;
            let mut base = raw_base;

            // Extend the register numbers with REX.X (index) and REX.B (base).
            if rex.contains(RexPrefixLow::INDEX) {
                index.set_bit(3, true);
            }
            if rex.contains(RexPrefixLow::BASE) {
                base.set_bit(3, true);
            }

            let mut addr = 0u64;

            // A SIB base field of 101b with mod=0 means "no base register, use a
            // disp32" -- selected from the raw 3-bit field, independent of REX.B.
            let no_base = self._mod == 0 && raw_base == 5;
            if !no_base {
                addr = as_reg(base)?.read()?;
            }

            // A raw index field of 100b means "no index register" only when REX.X
            // is clear; with REX.X it selects R12. The test therefore uses the
            // REX.X-extended value, which equals 4 only in the no-index case.
            if index != 4 {
                addr = addr.wrapping_add(as_reg(index)?.read()?.wrapping_mul(1u64 << scale));
            }

            // Add the displacement implied by the mod field.
            let sib_offset = 1; // skip the SIB byte
            match self._mod {
                0 => {
                    if no_base {
                        let mut buf = [0u8; 4];
                        buf.copy_from_slice(inst_slice(
                            inst,
                            disp_id + sib_offset..disp_id + sib_offset + 4,
                        )?);
                        addr = addr.wrapping_add(i32::from_ne_bytes(buf) as i64 as u64);
                    }
                }
                1 => {
                    let mut buf = [0u8; 1];
                    buf.copy_from_slice(inst_slice(
                        inst,
                        disp_id + sib_offset..disp_id + sib_offset + 1,
                    )?);
                    addr = addr.wrapping_add(i8::from_ne_bytes(buf) as i64 as u64);
                }
                2 => {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(inst_slice(
                        inst,
                        disp_id + sib_offset..disp_id + sib_offset + 4,
                    )?);
                    addr = addr.wrapping_add(i32::from_ne_bytes(buf) as i64 as u64);
                }
                _ => {}
            }

            let len = match self._mod {
                0 => {
                    if no_base {
                        4 + sib_offset
                    } else {
                        sib_offset
                    }
                }
                1 => 1 + sib_offset,
                2 => 4 + sib_offset,
                _ => sib_offset,
            };

            return Ok(MemOperand {
                gpa: gva_to_gpa(addr as _)?,
                len,
            });
        }

        // Non-SIB ModRM addressing
        let mut rm_base = self.rm;
        // REX.B extends the base register (but special case mod=0, rm=5 is RIP-relative, unaffected by REX.B)
        if rex.contains(RexPrefixLow::BASE) && !(self._mod == 0 && self.rm == 5) {
            rm_base.set_bit(3, true);
        }
        let reg = as_reg(rm_base)?;

        match self._mod {
            0 => {
                // In 64-bit mode mod=0, rm=5 is RIP-relative: the disp32 is added
                // to the address of the next instruction. For the MOV forms decoded
                // here the displacement is the final operand byte, so the
                // instruction ends at disp_id + 4. This form does not use the base
                // register, so it is not read here.
                if self.rm == 5 {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(inst_slice(inst, disp_id..disp_id + 4)?);
                    // mod=0, rm=5 is RIP-relative (signed disp32 from the next
                    // instruction) only in 64-bit mode; in 32-bit modes the same
                    // encoding is an absolute disp32.
                    let gva = if guest_is_64bit()? {
                        let disp = i32::from_ne_bytes(buf) as i64 as u64;
                        let next_rip =
                            (VmcsGuestNW::RIP.read()? as u64).wrapping_add((disp_id + 4) as u64);
                        next_rip.wrapping_add(disp)
                    } else {
                        u32::from_ne_bytes(buf) as u64
                    };
                    Ok(MemOperand {
                        gpa: gva_to_gpa(gva as _)?,
                        len: 4,
                    })
                } else {
                    Ok(MemOperand {
                        gpa: gva_to_gpa(reg.read()? as _)?,
                        len: 0,
                    })
                }
            }
            1 => {
                let mut buf = [0u8; 1];
                buf.copy_from_slice(inst_slice(inst, disp_id..disp_id + 1)?);
                let disp_8 = i8::from_ne_bytes(buf) as i64 as u64;
                let reg_val = reg.read()?.wrapping_add(disp_8);
                Ok(MemOperand {
                    gpa: gva_to_gpa(reg_val as _)?,
                    len: 1,
                })
            }
            2 => {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(inst_slice(inst, disp_id..disp_id + 4)?);
                let disp_32 = i32::from_ne_bytes(buf) as i64 as u64;
                let reg_val = reg.read()?.wrapping_add(disp_32);
                Ok(MemOperand {
                    gpa: gva_to_gpa(reg_val as _)?,
                    len: 4,
                })
            }
            // mod=3 is a register-direct operand, i.e. not a memory access, so it
            // cannot be the operand that faulted on MMIO. Reject it rather than
            // decode a register here -- this also avoids mis-decoding a legacy
            // high-byte register (AH/CH/DH/BH) named by a no-REX byte r/m field.
            3 => hv_result_err!(ENOSYS, "register-direct operand is not an MMIO access"),
            _ => hv_result_err!(EFAULT, "invalid modrm mod field"),
        }
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
    let cr0 = VmcsGuestNW::CR0.read()?;
    let cr4 = VmcsGuestNW::CR4.read()?;

    // Outside 64-bit mode the linear address is 32 bits, so mask the effective
    // address: a sign-extended disp32 (e.g. an absolute MMIO address with bit 31
    // set) or a 64-bit register's high bits must not leak into the translation.
    // Real mode is handled by the segment math below, not here.
    let gva = if cr0 & Cr0::CR0_PROTECTED_MODE.bits() != 0 && !cs_l_d()?.0 {
        gva & 0xffff_ffff
    } else {
        gva
    };

    // Paging off: under flat segmentation the linear address equals the physical
    // address. Real mode is segment-relative and a data operand's segment (DS/SS)
    // differs from CS; this minimal decoder does not model segmentation, so reject
    // real mode rather than translate a data operand against the wrong segment. A
    // modern guest only touches device MMIO in flat protected/long mode.
    if cr0 & Cr0::CR0_ENABLE_PAGING.bits() == 0 {
        if cr0 & Cr0::CR0_PROTECTED_MODE.bits() == 0 {
            return hv_result_err!(ENOSYS, "real-mode mmio translation not supported");
        }
        // Protected mode, paging off: only a flat 32-bit model -- a 32-bit default
        // operand size (CS.D = 1) with zero data and stack segment bases -- maps
        // the linear address straight to the physical address. Reject 16-bit or
        // non-flat segmentation rather than translate against an unmodelled base.
        let cs_d = cs_l_d()?.1;
        let flat = cs_d && VmcsGuestNW::DS_BASE.read()? == 0 && VmcsGuestNW::SS_BASE.read()? == 0;
        if !flat {
            return hv_result_err!(ENOSYS, "non-flat/16-bit protected-mode mmio not supported");
        }
        return Ok(gva);
    }

    if cr4 & Cr4::CR4_ENABLE_PAE.bits() == 0 {
        // 32-bit (non-PAE) protected-mode page walks are not implemented; report
        // it as unsupported so the caller injects a fault into the guest instead
        // of bringing the hypervisor down.
        return hv_result_err!(ENOSYS, "protected-mode (non-PAE) gva_to_gpa");
    }

    // Only long-mode (IA-32e) 4-level paging is walked here. A 32-bit PAE guest
    // uses a 3-level hierarchy, so detect long mode via EFER.LMA and reject other
    // PAE configurations rather than walking the wrong structure.
    if VmcsGuest64::IA32_EFER.read()? & (1 << 10) == 0 {
        return hv_result_err!(ENOSYS, "32-bit PAE paging not supported");
    }

    // In long mode the walk below is correct for 64-bit code (CS.L=1), whose data
    // segments are flat. Compatibility mode (CS.L=0) still applies the DS/SS
    // segment bases to a data operand, which this decoder does not model, so require
    // a flat 32-bit model (CS.D=1, zero data and stack bases) and reject otherwise
    // rather than translate against an unmodelled base.
    let (cs_l, cs_d) = cs_l_d()?;
    if !cs_l {
        let flat = cs_d && VmcsGuestNW::DS_BASE.read()? == 0 && VmcsGuestNW::SS_BASE.read()? == 0;
        if !flat {
            return hv_result_err!(
                ENOSYS,
                "non-flat/16-bit compatibility-mode mmio not supported"
            );
        }
    }

    // Each level's entry must be present; a non-present entry means the guest
    // virtual address is not mapped, which is a fault rather than a translation.
    let present = |entry: usize| -> HvResult<usize> {
        if entry & 1 == 0 {
            hv_result_err!(EFAULT, "non-present guest paging entry")
        } else {
            Ok(entry)
        }
    };
    let huge = DescriptorAttr::HUGE_PAGE.bits() as usize;

    let p4_gpa = (VmcsGuestNW::CR3.read()?) & !(0xfff);
    let p4_hpa = gpa_to_hpa(p4_gpa)?;
    let p4_entry_id = (gva >> 39) & 0x1ff;
    let p4_entry = present(get_page_entry(p4_hpa, p4_entry_id))?;

    let p3_gpa = p4_entry & !(0xfff);
    let p3_entry_id = (gva >> 30) & 0x1ff;
    let p3_hpa = gpa_to_hpa(p3_gpa)?;
    let p3_entry = present(get_page_entry(p3_hpa, p3_entry_id))?;

    if p3_entry & huge != 0 {
        // 1 GiB page: the base is 1 GiB-aligned, so mask off the full 30-bit
        // page offset (not just the 4 KiB bits) before adding it back from the gva.
        return Ok((p3_entry & !0x3fff_ffff) | (gva & 0x3fff_ffff));
    }

    let p2_gpa = p3_entry & !(0xfff);
    let p2_entry_id = (gva >> 21) & 0x1ff;
    let p2_hpa = gpa_to_hpa(p2_gpa)?;
    let p2_entry = present(get_page_entry(p2_hpa, p2_entry_id))?;

    if p2_entry & huge != 0 {
        // 2 MiB page: the base is 2 MiB-aligned, so mask off the full 21-bit
        // page offset before adding it back from the gva.
        return Ok((p2_entry & !0x1f_ffff) | (gva & 0x1f_ffff));
    }

    let p1_gpa = p2_entry & !(0xfff);
    let p1_entry_id = (gva >> 12) & 0x1ff;
    let p1_hpa = gpa_to_hpa(p1_gpa)?;
    let p1_entry = present(get_page_entry(p1_hpa, p1_entry_id))?;

    let page_gpa: usize = p1_entry & !(0xfff);
    Ok(page_gpa | (gva & 0xfff))
}

// Read the cached CS long-mode (L) and default-size (D/B) flags from the VMCS
// access-rights field. This reflects the CS that is actually loaded and avoids
// re-walking a guest-controlled GDT, whose GDTR may have changed since the load
// and whose descriptor read could fault or cross a page.
fn cs_l_d() -> HvResult<(bool, bool)> {
    let ar = VmcsGuest32::CS_ACCESS_RIGHTS.read()?;
    Ok((ar.get_bit(13), ar.get_bit(14)))
}

// Whether the guest is executing in 64-bit mode (CS.L = 1, CS.D = 0), which is
// the only mode where ModRM `mod=0, rm=5` is RIP-relative; otherwise it is an
// absolute disp32.
fn guest_is_64bit() -> HvResult<bool> {
    let cr0 = VmcsGuestNW::CR0.read()?;
    if cr0 & Cr0::CR0_PROTECTED_MODE.bits() == 0 {
        return Ok(false);
    }
    let (cs_l, cs_d) = cs_l_d()?;
    Ok(cs_l && !cs_d)
}

fn get_default_operand_size() -> HvResult<usize> {
    let cr0 = VmcsGuestNW::CR0.read()?;
    let mut size = size_of::<u16>();

    // in protection mode
    if cr0 & Cr0::CR0_PROTECTED_MODE.bits() != 0 {
        let (cs_l, cs_d) = cs_l_d()?;
        // 32-bit default operand size in 64-bit long mode or when CS.D is set.
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
    region_start: usize,
    region_len: usize,
    arg: usize,
) -> HvResult<usize> {
    // The fetch loop guarantees at least one byte (or already returned an error),
    // and every read below is bounds-checked, so no length assertion is needed.
    let mut size = get_default_operand_size()?;
    let mut cur_id = 0;

    // Consume legacy prefixes. Only the operand-size override (0x66) is modelled;
    // any other legacy prefix (address-size 0x67, a segment override, lock, or a
    // repeat prefix) would change the effective address or the semantics in ways
    // this minimal MOV/MOVZX decoder does not implement, so reject it -- the
    // access is then delivered to the guest as #UD rather than mis-decoded.
    // Repeated 0x66 prefixes are one effective override, so only a flag is kept.
    let mut operand_size_override = false;
    loop {
        match inst_byte(inst, cur_id)? {
            OPERAND_SIZE_OVERRIDE_PREFIX => {
                operand_size_override = true;
                cur_id += 1;
            }
            0x67 | 0x26 | 0x2e | 0x36 | 0x3e | 0x64 | 0x65 | 0xf0 | 0xf2 | 0xf3 => {
                return hv_result_err!(ENOSYS, "unsupported legacy prefix in mmio instruction");
            }
            _ => break,
        }
    }

    // REX must directly precede the opcode; if several REX bytes appear in a row
    // the last one is the effective prefix.
    let mut rex = RexPrefixLow::from_bits_truncate(0);
    let mut has_rex = false;
    while inst_byte(inst, cur_id)?.get_bits(4..=7) == REX_PREFIX_HIGH {
        rex = RexPrefixLow::from_bits_truncate(inst_byte(inst, cur_id)?.get_bits(0..=3));
        has_rex = true;
        cur_id += 1;
    }

    // REX.W (64-bit operand) takes precedence over the 0x66 override; otherwise the
    // 0x66 override flips between the default 32-bit and 16-bit operand size once.
    if rex.contains(RexPrefixLow::OPERAND_WIDTH) {
        size = size_of::<u64>();
    } else if operand_size_override {
        size = if size == size_of::<u32>() {
            size_of::<u16>()
        } else {
            size_of::<u32>()
        };
    }

    let mut two_byte = false;
    if inst_byte(inst, cur_id)? == TWO_BYTE_ESCAPE {
        two_byte = true;
        cur_id += 1;
    }

    if !two_byte {
        let opcode: OneByteOpCode = match inst_byte(inst, cur_id)?.try_into() {
            Ok(op) => op,
            Err(_) => {
                return hv_result_err!(ENOSYS, format!("unimplemented mmio opcode: {:#x?}", inst))
            }
        };
        cur_id += 1;

        // The memory operand width comes from the opcode: a byte MOV always
        // accesses one byte (the 0x66 prefix and REX.W only change the register
        // width), while a word/dword/qword MOV uses the prefix-adjusted size.
        size = match opcode {
            OneByteOpCode::MovEbGb | OneByteOpCode::MovGbEb => size_of::<u8>(),
            _ => size,
        };

        match opcode {
            OneByteOpCode::MovEbGb | OneByteOpCode::MovEvGv => {
                let mod_rm = ModRM::new(inst_byte(inst, cur_id)?, &rex);
                cur_id += 1;

                // A byte operand without a REX prefix using register code 4-7 names
                // a legacy high-byte register (AH/CH/DH/BH), which this decoder does
                // not model; reject it rather than touch the wrong register.
                if size == 1 && !has_rex && mod_rm.reg_opcode >= 4 {
                    return hv_result_err!(ENOSYS, "high-byte register operand");
                }

                let src = mod_rm.get_reg();
                let src_val = src.read()?;

                let MemOperand { gpa, len } = mod_rm.get_modrm(inst, cur_id, &rex)?;
                cur_id += len;

                mmio.address = region_offset(gpa, region_start, region_len, size)?;
                mmio.is_write = true;
                mmio.size = size;
                // Pass only the bytes actually stored: a sub-register write
                // (MOV r/m8,r8 etc.) must not leak the source register's upper bits
                // to a device handler that does not itself mask by `size`.
                mmio.value = src_val.get_bits(0..(size * 8)) as _;

                handler(mmio, arg)?;

                Ok(cur_id)
            }
            OneByteOpCode::MovGbEb | OneByteOpCode::MovGvEv => {
                let mod_rm = ModRM::new(inst_byte(inst, cur_id)?, &rex);
                cur_id += 1;

                // See above: a high-byte register (AH/CH/DH/BH) operand is rejected.
                if size == 1 && !has_rex && mod_rm.reg_opcode >= 4 {
                    return hv_result_err!(ENOSYS, "high-byte register operand");
                }

                let dst = mod_rm.get_reg();

                let MemOperand { gpa, len } = mod_rm.get_modrm(inst, cur_id, &rex)?;
                cur_id += len;

                mmio.address = region_offset(gpa, region_start, region_len, size)?;
                mmio.is_write = false;
                mmio.size = size;
                mmio.value = 0;
                handler(mmio, arg)?;
                let src_val = mmio.value as u64;

                dst.write(src_val, size)?;
                Ok(cur_id)
            }
            _ => {
                hv_result_err!(
                    ENOSYS,
                    format!("Unimplemented opcode: 0x{:x}", opcode as u8)
                )
            }
        }
    } else {
        let opcode: TwoByteOpCode = match inst_byte(inst, cur_id)?.try_into() {
            Ok(op) => op,
            Err(_) => {
                return hv_result_err!(ENOSYS, format!("unimplemented mmio opcode: {:#x?}", inst))
            }
        };
        cur_id += 1;

        // MOVZX reads a fixed-width memory source (byte or word) and zero-extends
        // it into a destination register whose width is the prefix-selected
        // operand size, so the two widths are tracked separately.
        let mem_size = match opcode {
            TwoByteOpCode::MovZxGvEb => size_of::<u8>(),
            TwoByteOpCode::MovZxGvEw => size_of::<u16>(),
            _ => size,
        };

        match opcode {
            TwoByteOpCode::MovZxGvEb | TwoByteOpCode::MovZxGvEw => {
                let mod_rm = ModRM::new(inst_byte(inst, cur_id)?, &rex);
                cur_id += 1;

                let dst = mod_rm.get_reg();

                let MemOperand { gpa, len } = mod_rm.get_modrm(inst, cur_id, &rex)?;
                cur_id += len;

                mmio.address = region_offset(gpa, region_start, region_len, mem_size)?;
                mmio.is_write = false;
                mmio.size = mem_size;
                mmio.value = 0;
                handler(mmio, arg)?;
                let src_val = mmio.value as u64;

                let src_val_zero_extend = match mem_size {
                    1 => src_val.get_bits(0..8),
                    2 => src_val.get_bits(0..16),
                    _ => src_val,
                };

                dst.write(src_val_zero_extend, size)?;
                Ok(cur_id)
            }
            _ => {
                hv_result_err!(
                    ENOSYS,
                    format!("Unimplemented opcode: 0x{:x}", opcode as u8)
                )
            }
        }
    }
}

// Translate a decoded MMIO guest-physical access into a device-relative offset,
// rejecting any access that is not fully contained in the faulting region so a
// straddling or below-base address can never underflow or escape the handler. The
// unmapped/no-device path passes `base = 0` and `region_len = usize::MAX`, i.e.
// the whole address space, so any access whose end does not overflow passes.
fn region_offset(gpa: usize, base: usize, region_len: usize, size: usize) -> HvResult<usize> {
    let contained = gpa >= base
        && gpa
            .checked_add(size)
            .zip(base.checked_add(region_len))
            .map_or(false, |(acc_end, reg_end)| acc_end <= reg_end);
    if contained {
        Ok(gpa - base)
    } else {
        hv_result_err!(EFAULT, "mmio access outside the faulting region")
    }
}

#[test_case]
fn test_mmio_region_offset_requires_full_containment() {
    assert_eq!(region_offset(0x1080, 0x1000, 0x100, 4).unwrap(), 0x80);
    assert!(region_offset(0x0fff, 0x1000, 0x100, 1).is_err());
    assert!(region_offset(0x10ff, 0x1000, 0x100, 2).is_err());
    assert!(region_offset(usize::MAX - 1, 0, usize::MAX, 4).is_err());
}

#[test_case]
fn test_mmio_instruction_fetch_helpers_are_bounds_checked() {
    let inst = [0x48, 0x8b, 0x05, 0x01];
    assert_eq!(inst_byte(&inst, 1).unwrap(), 0x8b);
    assert_eq!(inst_slice(&inst, 1..3).unwrap(), &[0x8b, 0x05]);
    assert!(inst_byte(&inst, inst.len()).is_err());
    assert!(inst_slice(&inst, 2..6).is_err());
}

#[test_case]
fn test_mmio_modrm_rex_extends_register_field() {
    let rex = RexPrefixLow::REGISTERS;
    let modrm = ModRM::new(0b0000_0000, &rex);
    assert_eq!(modrm.reg_opcode, RmReg::R8 as u32);
}

pub fn instruction_emulator(
    handler: &MMIOHandler,
    mmio: &mut MMIOAccess,
    region_start: usize,
    region_len: usize,
    arg: usize,
) -> HvResult {
    // An x86 instruction is at most 15 bytes and may straddle a page boundary, so
    // fetch it page by page through guest translation rather than reading 15
    // host-contiguous bytes from the first page (which need not be contiguous in
    // host memory, nor even mapped, past the page edge). The page holding RIP must
    // translate; fetching stops at the first unmapped continuation page and the
    // decoder reads the buffer through bounds-checked accessors, so an instruction
    // that would need an unfetched byte is reported as an error (a guest #UD)
    // rather than panicking.
    const MAX_INST_LEN: usize = 15;
    const PAGE_SIZE: u64 = 0x1000;
    let mut inst = Vec::with_capacity(MAX_INST_LEN);
    let mut gva = VmcsGuestNW::RIP.read()? as u64;
    while inst.len() < MAX_INST_LEN {
        let hpa = match gva_to_gpa(gva as usize).and_then(gpa_to_hpa) {
            Ok(hpa) => hpa as *const u8,
            // The page holding RIP must translate; a later page that does not is
            // simply not part of this instruction.
            Err(_) if !inst.is_empty() => break,
            Err(e) => return Err(e),
        };
        let in_page = (PAGE_SIZE - (gva & (PAGE_SIZE - 1))) as usize;
        let take = in_page.min(MAX_INST_LEN - inst.len());
        inst.extend_from_slice(unsafe { from_raw_parts(hpa, take) });
        // RIP can sit in the top page of the address space; advancing past the end
        // would wrap. `inst` is already non-empty here, so stop and let the decoder
        // report a truncated instruction (guest #UD) rather than overflow.
        gva = match gva.checked_add(take as u64) {
            Some(next) => next,
            None => break,
        };
    }

    let len = emulate_inst(&inst, handler, mmio, region_start, region_len, arg)?;

    this_cpu_data().arch_cpu.advance_guest_rip(len as _)?;

    Ok(())
}

pub fn mmio_empty_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    if !mmio.is_write {
        mmio.value = 0;
    }
    Ok(())
}
