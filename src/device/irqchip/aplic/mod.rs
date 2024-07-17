
use spin::RwLock;
use spin::Once;
use crate::{arch::cpu::ArchCpu, percpu::this_cpu_data, memory::GuestPhysAddr};

use riscv_decode::Instruction;
use fdt::Fdt;
use crate::zone::Zone;

// S-mode interrupt delivery controller
const APLIC_S_IDC: usize = 0xd00_4000;
pub const APLIC_DOMAINCFG_BASE: usize = 0x0000;
pub const APLIC_SOURCECFG_BASE: usize = 0x0004;
pub const APLIC_SOURCECFG_TOP: usize = 0x1000;
pub const APLIC_MSIADDR_BASE: usize = 0x1BC8;
pub const APLIC_PENDING_BASE: usize = 0x1C00;
pub const APLIC_PENDING_TOP: usize = 0x1C80;
pub const APLIC_CLRIP_BASE: usize = 0x1D00;
pub const APLIC_ENABLE_BASE: usize = 0x1E00;
pub const APLIC_ENABLE_TOP: usize = 0x1E7C;
pub const APLIC_ENABLE_NUM: usize = 0x1EDC;
pub const APLIC_CLRIE_BASE: usize = 0x1F00;
pub const APLIC_CLRIE_NUM_BASE: usize = 0x1FDC;
pub const APLIC_IPNUM_LE_BASE: usize = 0x2000;
pub const APLIC_TARGET_BASE: usize = 0x3004;
pub const APLIC_IDC_BASE: usize = 0x4000;

#[repr(u32)]
#[allow(dead_code)]
pub enum SourceModes { 
    Inactive = 0,
    Detached = 1,
    RisingEdge = 4,
    FallingEdge = 5,
    LevelHigh = 6,
    LevelLow = 7,
}

// offset size register name
// 0x0000 4 bytes domaincfg
// 0x0004 4 bytes sourcecfg[1]
// 0x0008 4 bytes sourcecfg[2]
// . . . . . .
// 0x0FFC 4 bytes sourcecfg[1023]
// 0x1BC0 4 bytes mmsiaddrcfg (machine-level interrupt domains only)
// 0x1BC4 4 bytes mmsiaddrcfgh ”
// 0x1BC8 4 bytes smsiaddrcfg ”
// 0x1BCC 4 bytes smsiaddrcfgh ”
// 0x1C00 4 bytes setip[0]
// 0x1C04 4 bytes setip[1]
// . . . . . .
// 0x1C7C 4 bytes setip[31]
// 0x1CDC 4 bytes setipnum
// 0x1D00 4 bytes in clrip[0]
// 0x1D04 4 bytes in clrip[1]
// . . . . . .
// 0x1D7C 4 bytes in clrip[31]
// 0x1DDC 4 bytes clripnum
// 0x1E00 4 bytes setie[0]
// 0x1E04 4 bytes setie[1]
// . . . . . .
// 0x1E7C 4 bytes setie[31]
// 0x1EDC 4 bytes setienum
// 0x1F00 4 bytes clrie[0]
// 0x1F04 4 bytes clrie[1]
// . . . . . .
// 0x1F7C 4 bytes clrie[31]
// 0x1FDC 4 bytes clrienum
// 0x2000 4 bytes setipnum le
// 0x2004 4 bytes setipnum be
// 0x3000 4 bytes genmsi
// 0x3004 4 bytes target[1]
// 0x3008 4 bytes target[2]
// . . . . . .
// 0x3FFC 4 bytes target[1023]

pub fn primary_init_early(host_fdt: &Fdt) {
    let aplic_debug = host_fdt.find_node("/soc/aplic").unwrap();
    init_aplic(
        aplic_debug.reg().unwrap().next().unwrap().starting_address as usize,
        aplic_debug.reg().unwrap().next().unwrap().size.unwrap(),
    );
}
pub fn primary_init_late() {
    //nothing to do
}
pub fn percpu_init() {
    //nothing to do
}
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    //nothing to do
}
pub static APLIC: Once<RwLock<Aplic>> = Once::new();
pub fn host_aplic<'a>() -> &'a RwLock<Aplic> {
    APLIC.get().expect("Uninitialized hypervisor aplic!")
}

#[repr(C)]
pub struct Aplic {
    pub base: usize,
    pub size: usize,
}

#[allow(dead_code)]
impl Aplic {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
        }
    }
    pub fn set_domaincfg(&self, bigendian: bool, msimode: bool, enabled: bool){
        let enabled = u32::from(enabled);
        let msimode = u32::from(msimode);
        let bigendian = u32::from(bigendian);
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let bigendian = 0 ;
        let src = (enabled << 8) | (msimode << 2) | bigendian;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn read_domaincfg(&self) -> u32{
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn get_msimode(&self) -> bool{
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let value= unsafe { core::ptr::read_volatile(addr as *const u32) };
        ((value >> 2) & 0b11) != 0
    }
    pub fn set_sourcecfg(&self, irq: u32, mode: SourceModes){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = mode as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn set_sourcecfg_delegate(&self, irq: u32, child: u32){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = 1 << 10 | child & 0x3ff;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn read_sourcecfg(&self, irq: u32) -> u32{
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_msiaddr(&self, address: usize){
        let addr = self.base + APLIC_MSIADDR_BASE;
        let src = (address >> 12) as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
            core:: ptr::write_volatile((addr + 4) as *mut u32, 0);
        }
    }
    pub fn read_pending(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn read_clr_pending(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_pending(&self, irqidx: usize, value: u32, pending: bool){
        assert!(irqidx < 32);
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        if pending {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, value);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, value);
            }
        }
    } 
    pub fn read_in_clrip(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn read_enable(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn read_clr_enable(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_enable(&self, irqidx: usize, value: u32, enabled: bool){
        assert!(irqidx < 32);
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        if enabled {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, value);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, value);
            }
        }
    } 
    pub fn set_enable_num(&self, value: u32){
        let addr = self.base + APLIC_ENABLE_NUM;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_clr_ienum(&self, value: u32){
        let addr = self.base + APLIC_CLRIE_NUM_BASE;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_ipnum_le(&self, value: u32){
        let addr = self.base + APLIC_IPNUM_LE_BASE;
        // let value_le = u32::from_le_bytes(value.to_be_bytes());
        debug!("value:0x{:08x}", value);
        // debug!("value_le:0x{:08x}", value_le);
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_target_msi(&self, irq: u32, hart: u32, guest: u32, eiid: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src = (hart << 18) | (guest << 12) | eiid;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn set_target_direct(&self, irq: u32, hart: u32, prio: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src =  (hart << 18) | (prio & 0xFF);
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
}

pub fn vaplic_emul_handler(
    current_cpu: &mut ArchCpu,
    addr: GuestPhysAddr,
    inst: Instruction,
) {
    let host_aplic = host_aplic();
    let offset = addr.wrapping_sub(host_aplic.read().base);
    if offset >= APLIC_DOMAINCFG_BASE && offset < APLIC_SOURCECFG_BASE {
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;      // 要写入的 value
                let enabled = ((value >> 8) & 0x1) != 0;                // IE
                let msimode = ((value >> 2) & 0b1) != 0;                // DM / MSI
                let bigendian = (value & 0b1) != 0;                     // 大小端
                host_aplic.write().set_domaincfg(bigendian, msimode, enabled);
                debug!(
                    "APLIC set domaincfg write addr@{:#x} bigendian {} msimode {} enabled {}",
                    addr, bigendian, msimode, enabled
                );
            }
            Instruction::Lw(i) => {                                     // 直接读取对应的内容
                let value = host_aplic.read().read_domaincfg();
                current_cpu.x[i.rd() as usize] = value as usize;
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }  
    else if offset >= APLIC_SOURCECFG_BASE && offset < APLIC_SOURCECFG_TOP {
        //sourcecfg
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irq = ((offset - APLIC_SOURCECFG_BASE) / 4) + 1;
                if (value >> 10) & 0b1 == 1 {
                    //delegate
                    let child = value & 0x3ff;
                    host_aplic.write().set_sourcecfg_delegate(irq as u32, child);
                    debug!(
                        "APLIC set sourcecfg_delegate write addr@{:#x} irq {} child {}",
                        addr,
                        irq,
                        child
                    );
                } else {    
                    let mode = match value {
                        0 => SourceModes::Inactive,
                        1 => SourceModes::Detached,
                        4 => SourceModes::RisingEdge,
                        5 => SourceModes::FallingEdge,
                        6 => SourceModes::LevelHigh,
                        7 => SourceModes::LevelLow,
                        _ => panic!("Unknown sourcecfg mode"),
                    };
                    host_aplic.write().set_sourcecfg(irq as u32, mode);
                    debug!(
                        "APLIC set sourcecfg write addr@{:#x} irq {} mode {}",
                        addr,
                        irq,
                        value
                    );
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_MSIADDR_BASE && offset <= 0x1BCC {
        // msia
         match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let address = (value as usize) << 12;
                host_aplic.write().set_msiaddr(address);
                debug!(
                    "APLIC set msiaddr write addr@{:#x} address {}",
                    addr, address
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_PENDING_BASE && offset < APLIC_PENDING_TOP {
        // pending
        panic!("setip Unexpected instruction {:?}", inst);
    } 
    // setipnum 区域        0x1CDC  -  0x1CE0
    else if offset >= 0x1CDC && offset < 0x1CE0 {
        panic!("setipnum Unexpected instruction {:?}", inst)
    }
    else if offset >= APLIC_CLRIP_BASE && offset < 0x1D80 {
        // panic!("addr@{:#x} in_clrip Unexpected instruction {:?}", offset ,inst);
        match inst {
            Instruction::Lw(i) => {
                let irqidx = (offset - APLIC_CLRIP_BASE) / 4;
                let value = host_aplic.read().read_in_clrip(irqidx);
                current_cpu.x[i.rd() as usize] = value as usize;
                debug!(
                    "APLIC read in clrip addr@{:#x} irqidx {} value {}",
                    addr, irqidx, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }
    // clripnum 区域
    else if offset >= 0x1DDC && offset < 0x1DE0 {
        panic!("clripnum Unexpected instruction {:?}", inst)
    }
    // setie
    else if offset >= APLIC_ENABLE_BASE && offset < 0x1E80 {
        panic!("setie Unexpected instruction {:?}", inst);
    }  
    else if offset >= APLIC_ENABLE_NUM && offset < 0x1EE0 {
        // enablenum
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_aplic.write().set_enable_num(value);
                debug!(
                    "APLIC set enablenum write addr@{:#x} value {}",
                    addr, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_CLRIE_BASE && offset < 0x1FDC {
        // clrenable
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irqidx = (offset - APLIC_CLRIE_BASE) / 4;
                host_aplic.write().set_enable(irqidx, value, false);
                debug!(
                    "APLIC set clr_enable write addr@{:#x} irqidx {} value {}",
                    addr, irqidx, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }  
    // clrienum
    else if offset >= 0x1FDC && offset < 0x1FE0 {
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_aplic.write().set_clr_ienum(value);
                debug!(
                    "APLIC set set_clr_ienum write addr@{:#x} value@{:#x}",
                    offset, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
        
    }
    // setipnum_le
    else if offset >= 0x2000 && offset < 0x2004 {
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_aplic.write().set_ipnum_le(value);
                debug!(
                    "APLIC set set_ipnum_le write addr@{:#x} value@{:#x}",
                    offset, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }    
    // setipnum_be
    else if offset >= 0x2004 && offset < 0x2008 {
        panic!("setipnum_be Unexpected instruction {:?}", inst)
    }
    // genmsi 
    else if offset >= 0x3000 && offset < 0x3004 {
        panic!("genmsi Unexpected instruction {:?}", inst)
    }
    else if offset >= APLIC_TARGET_BASE && offset < APLIC_IDC_BASE {
        // target
        match inst {
            Instruction::Sw(i) => {
                let first_cpu = this_cpu_data()
                .zone
                .as_ref()
                .unwrap()
                .read()
                .cpu_set
                .first_cpu()
                .unwrap();
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irq = ((offset - APLIC_TARGET_BASE) / 4) as u32 + 1;
                let hart = ((value >> 18) & 0x3F) + first_cpu as u32;
                if host_aplic.read().get_msimode() {
                    let guest = ((value >> 12) & 0x3F) + 1;
                    let eiid = value & 0xFFF;
                    host_aplic.write().set_target_msi(irq, hart, guest, eiid);
                    debug!(
                        "APLIC set msi target write addr@{:#x} irq {} hart {} guest {} eiid {}",
                        addr, irq, hart, guest, eiid
                    );
                } else {
                    let prio = value & 0xFF;
                    host_aplic.write().set_target_direct(irq, hart, prio);
                    debug!(
                        "APLIC set direct target write addr@{:#x} irq {} hart {} prio {}",
                        addr, irq, hart, prio
                    );
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
pub fn init_aplic(aplic_base: usize, aplic_size: usize) {
    let aplic = Aplic::new(aplic_base, aplic_size);
    APLIC.call_once(|| RwLock::new(aplic));
}
impl Zone {
    pub fn arch_irqchip_reset(&self) {
        //TODO
    }
}