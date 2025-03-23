use crate::{
    arch::{
        s2pt::DescriptorAttr,
        vmcs::{VmcsGuest16, VmcsGuestNW},
    },
    error::HvResult,
    memory::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr, MMIOAccess},
    percpu::{this_cpu_data, this_zone},
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{mem::size_of, ops::Range, slice::from_raw_parts};
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
        value.set_bits(0..(size * 4), new_value.get_bits(0..(size * 4)));

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
B: byte
V: word / dword / qword
*/
numeric_enum_macro::numeric_enum! {
#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    // move r to r/m
    MovEvGv = 0x89,
    // move r/m to r
    MovGvEv = 0x8b,
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

// len stands for instruction len
enum OprandType {
    Reg { reg: RmReg, len: usize },
    Gpa { gpa: usize, len: usize },
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

    pub fn get_modrm(&self, inst: &Vec<u8>, disp_id: usize) -> Option<OprandType> {
        let reg: RmReg = self.rm.try_into().unwrap();
        let mut reg_val = reg.read().unwrap();
        // TODO: SIB
        match self._mod {
            0 => Some(OprandType::Gpa {
                gpa: gva_to_gpa(reg_val as _).unwrap(),
                len: 0,
            }),
            1 => {
                let mut buf = [0u8; 1];
                buf[0..1].copy_from_slice(&inst[disp_id..disp_id + 1]);
                let disp_8 = i8::from_ne_bytes(buf);
                if disp_8 > 0 {
                    reg_val += (disp_8 as u64);
                } else {
                    reg_val -= ((-disp_8) as u64);
                }
                Some(OprandType::Gpa {
                    gpa: gva_to_gpa(reg_val as _).unwrap(),
                    len: 1,
                })
            }
            2 => {
                let mut buf = [0u8; 4];
                buf[0..4].copy_from_slice(&inst[disp_id..disp_id + 4]);
                let disp_32 = i32::from_ne_bytes(buf);
                if disp_32 > 0 {
                    reg_val += (disp_32 as u64);
                } else {
                    reg_val -= ((-disp_32) as u64);
                }
                Some(OprandType::Gpa {
                    gpa: gva_to_gpa(reg_val as _).unwrap(),
                    len: 4,
                })
            }
            3 => Some(OprandType::Reg { reg, len: 0 }),
            _ => None,
        }
    }
}

fn gpa_to_hpa(gpa: GuestPhysAddr) -> HvResult<HostPhysAddr> {
    let (hpa, _, _) = unsafe { this_zone().read().gpm.page_table_query(gpa)? };
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

    let p4_gpa = VmcsGuestNW::CR3.read()?;
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

fn emulate_inst(inst: &Vec<u8>, dev: &Arc<dyn MMIoDevice>) -> HvResult<usize> {
    assert!(inst.len() > 0);

    let mut cur_id = 0;

    let mut rex = RexPrefixLow::from_bits_truncate(0);
    if inst[cur_id].get_bits(4..=7) == REX_PREFIX_HIGH {
        rex = RexPrefixLow::from_bits_truncate(inst[cur_id].get_bits(0..=3));
        assert!(rex == RexPrefixLow::REGISTERS);
        cur_id += 1;
    }

    let opcode: OpCode = inst[cur_id].try_into().unwrap();
    cur_id += 1;

    match opcode {
        OpCode::MovEvGv => {
            let mod_rm = ModRM::new(inst[cur_id], &rex);
            cur_id += 1;

            let src = mod_rm.get_reg();
            let src_val = src.read().unwrap();

            let dst = mod_rm.get_modrm(inst, cur_id).unwrap();
            match dst {
                OprandType::Reg { reg, len } => {
                    cur_id += len;
                    reg.write(src_val, size_of::<u32>()).unwrap();
                }
                OprandType::Gpa { gpa, len } => {
                    cur_id += len;
                    dev.write(gpa, src_val, size_of::<u32>()).unwrap();
                }
                _ => {}
            }

            Ok(cur_id)
        }
        OpCode::MovGvEv => {
            let mod_rm = ModRM::new(inst[cur_id], &rex);
            cur_id += 1;

            let dst = mod_rm.get_reg();

            let src = mod_rm.get_modrm(inst, cur_id).unwrap();
            let src_val = match src {
                OprandType::Reg { reg, len } => {
                    cur_id += len;
                    reg.read().unwrap()
                }
                OprandType::Gpa { gpa, len } => {
                    cur_id += len;
                    dev.read(gpa).unwrap()
                }
            };
            // info!("src_val: {:x}", src_val);

            dst.write(src_val, size_of::<u32>()).unwrap();
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

pub fn mmio_handler(mmio: &mut MMIOAccess, dev: &Arc<dyn MMIoDevice>) -> HvResult {
    let rip_hpa = gpa_to_hpa(gva_to_gpa(VmcsGuestNW::RIP.read()?)?)? as *const u8;
    let inst = unsafe { from_raw_parts(rip_hpa, 15) }.to_vec();

    // info!("rip_hpa: {:?}, inst: {:x?}", rip_hpa, inst);

    let len = emulate_inst(&inst, dev).unwrap();
    this_cpu_data().arch_cpu.advance_guest_rip(len as _)?;

    Ok(())
}
