use crate::{
    error::HvResult,
    memory::{
        addr::align_up, mmio_generic_handler, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
    },
    zone::Zone,
};
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{fence, Ordering};

impl Zone {
    pub fn pt_init(
        &mut self,
        vm_paddr_start: usize,
        fdt: &fdt::Fdt,
        guest_dtb: usize,
        dtb_ipa: usize,
    ) -> HvResult {
        info!("loongarch64: mm: pt init for zone, vm_paddr_start: {:#x?}, guest_dtb: {:#x?}, dtb_ipa: {:#x?}", vm_paddr_start, guest_dtb, dtb_ipa);
        Ok(())
    }

    pub fn mmio_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: mmio_init do nothing");
    }
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: isa_init do nothing");
    }
    pub fn irq_bitmap_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: irq_bitmap_init do nothing");
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LoongArch64VcpuContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 10],
}

fn prepare_vm_trapcontext(guest_entry_addr: usize, trap_addr: usize, vm_pagetable: usize) {
    unsafe {
        // guest entry address
        asm!("st.d {}, {}, 256", in(reg) guest_entry_addr, in(reg) trap_addr);
        // GCSRS
        asm!("gcsrrd $r12, 0x0");
        asm!("st.d $r12, {}, 256+8*1", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1");
        asm!("st.d $r12, {}, 256+8*2", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x2");
        asm!("st.d $r12, {}, 256+8*3", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3");
        asm!("st.d $r12, {}, 256+8*4", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x4");
        asm!("st.d $r12, {}, 256+8*5", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x5");
        asm!("st.d $r12, {}, 256+8*6", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x6");
        asm!("st.d $r12, {}, 256+8*7", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x7");
        asm!("st.d $r12, {}, 256+8*8", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8");
        asm!("st.d $r12, {}, 256+8*9", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0xc");
        asm!("st.d $r12, {}, 256+8*10", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x10");
        asm!("st.d $r12, {}, 256+8*11", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x11");
        asm!("st.d $r12, {}, 256+8*12", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x12");
        asm!("st.d $r12, {}, 256+8*13", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x13");
        asm!("st.d $r12, {}, 256+8*14", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x18");
        asm!("st.d $r12, {}, 256+8*15", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x19");
        asm!("st.d $r12, {}, 256+8*16", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1a");
        asm!("st.d $r12, {}, 256+8*17", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1b");
        asm!("st.d $r12, {}, 256+8*18", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1c");
        asm!("st.d $r12, {}, 256+8*19", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1d");
        asm!("st.d $r12, {}, 256+8*20", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1e");
        asm!("st.d $r12, {}, 256+8*21", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x1f");
        asm!("st.d $r12, {}, 256+8*22", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x20");
        asm!("st.d $r12, {}, 256+8*23", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x21");
        asm!("st.d $r12, {}, 256+8*24", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x22");
        asm!("st.d $r12, {}, 256+8*25", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x23");
        asm!("st.d $r12, {}, 256+8*26", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x30");
        asm!("st.d $r12, {}, 256+8*27", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x31");
        asm!("st.d $r12, {}, 256+8*28", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x32");
        asm!("st.d $r12, {}, 256+8*29", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x33");
        asm!("st.d $r12, {}, 256+8*30", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x34");
        asm!("st.d $r12, {}, 256+8*31", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x35");
        asm!("st.d $r12, {}, 256+8*32", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x36");
        asm!("st.d $r12, {}, 256+8*33", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x37");
        asm!("st.d $r12, {}, 256+8*34", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x38");
        asm!("st.d $r12, {}, 256+8*35", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x39");
        asm!("st.d $r12, {}, 256+8*36", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3a");
        asm!("st.d $r12, {}, 256+8*37", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3b");
        asm!("st.d $r12, {}, 256+8*38", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3c");
        asm!("st.d $r12, {}, 256+8*39", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3d");
        asm!("st.d $r12, {}, 256+8*40", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3e");
        asm!("st.d $r12, {}, 256+8*41", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x3f");
        asm!("st.d $r12, {}, 256+8*42", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x40");
        asm!("st.d $r12, {}, 256+8*43", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x41");
        asm!("st.d $r12, {}, 256+8*44", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x42");
        asm!("st.d $r12, {}, 256+8*45", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x43");
        asm!("st.d $r12, {}, 256+8*46", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x44");
        asm!("st.d $r12, {}, 256+8*47", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x60");
        asm!("st.d $r12, {}, 256+8*48", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x88");
        asm!("st.d $r12, {}, 256+8*49", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x89");
        asm!("st.d $r12, {}, 256+8*50", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8a");
        asm!("st.d $r12, {}, 256+8*51", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8b");
        asm!("st.d $r12, {}, 256+8*52", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8c");
        asm!("st.d $r12, {}, 256+8*53", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8d");
        asm!("st.d $r12, {}, 256+8*54", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8e");
        asm!("st.d $r12, {}, 256+8*55", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x8f");
        asm!("st.d $r12, {}, 256+8*56", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x180");
        asm!("st.d $r12, {}, 256+8*57", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x181");
        asm!("st.d $r12, {}, 256+8*58", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x182");
        asm!("st.d $r12, {}, 256+8*59", in(reg) trap_addr);
        asm!("gcsrrd $r12, 0x183");
        asm!("st.d $r12, {}, 256+8*60", in(reg) trap_addr);
        // vm pagetable address
        asm!("st.d {}, {}, 256+8*61", in(reg) vm_pagetable, in(reg) trap_addr);
        asm!("st.d {}, {}, 256+8*62", in(reg) vm_pagetable, in(reg) trap_addr);
    }
}

// pub fn first_sched_callback_fn() {
//     set_zone_trap_entry();
//     let cur_vcpu = current_vcpu().unwrap();
//     let trap_addr = cur_vcpu.get_kernel_stack_top();

//     let cur_zone = current_vcpu().unwrap().get_zone().unwrap();
//     let guest_entry_addr = cur_zone.get_sepc();

//     unsafe {
//         // gcsr_dump();
//         // prepare trap context
//         prepare_vm_trapcontext(guest_entry_addr, trap_addr, cur_zone.pagetable_dir());
//         let start_time: usize;
//         let counter_id = 0;
//         asm!("rdtime.d {}, {}", out(reg) start_time, in(reg) counter_id);
//         cur_zone.set_start_time(start_time);
//         zone_trap_ret_rust(trap_addr);
//     }
// }

pub fn first_sched_callback_fn() {
    // todo
}

impl LoongArch64VcpuContext {
    pub const fn new() -> LoongArch64VcpuContext {
        LoongArch64VcpuContext {
            ra: 0,
            sp: 0,
            s: [0; 10],
        }
    }
    pub fn first_sched_callback(kstack_ptr: usize) -> LoongArch64VcpuContext {
        LoongArch64VcpuContext {
            ra: first_sched_callback_fn as usize,
            sp: kstack_ptr,
            s: [0; 10],
        }
    }
    pub fn print_vcpu_context(&self) {
        info!("==============Vcpu Context============");
        info!("ra: {:#x}", self.ra);
        info!("sp: {:#x}", self.sp);
        info!("s[0]: {:#x}", self.s[0]);
        info!("s[1]: {:#x}", self.s[1]);
        info!("s[2]: {:#x}", self.s[2]);
        info!("s[3]: {:#x}", self.s[3]);
        info!("s[4]: {:#x}", self.s[4]);
        info!("s[5]: {:#x}", self.s[5]);
        info!("s[6]: {:#x}", self.s[6]);
        info!("s[7]: {:#x}", self.s[7]);
        info!("s[8]: {:#x}", self.s[8]);
        info!("s[9]: {:#x}", self.s[9]);
    }
}

#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct LoongArch64ZoneContext {
    pub x: [usize; 32],
    pub sepc: usize,
    // General Control and Status Registers
    pub gcsr_crmd: usize,   // CRMD
    pub gcsr_prmd: usize,   // PRMD
    pub gcsr_euen: usize,   // EUEN
    pub gcsr_misc: usize,   // MISC
    pub gcsr_ectl: usize,   // ECTL
    pub gcsr_estat: usize,  // ESTAT
    pub gcsr_era: usize,    // ERA
    pub gcsr_badv: usize,   // BADV
    pub gcsr_badi: usize,   // BADI
    pub gcsr_eentry: usize, // EENTRY

    // TLB Registers
    pub gcsr_tlbidx: usize,  // TLBIDX
    pub gcsr_tlbehi: usize,  // TLBEHI
    pub gcsr_tlbelo0: usize, // TLBELO0
    pub gcsr_tlbelo1: usize, // TLBELO1

    // Page Table Registers
    pub gcsr_asid: usize, // ASID
    pub gcsr_pgdl: usize, // PGDL
    pub gcsr_pgdh: usize, // PGDH
    pub gcsr_pgd: usize,  // PGD
    pub gcsr_pwcl: usize, // PWCL
    pub gcsr_pwch: usize, // PWCH

    // Second Level TLB Registers
    pub gcsr_stlbps: usize, // STLBPS
    pub gcsr_ravcfg: usize, // RAVCFG

    // Processor Registers
    pub gcsr_cpuid: usize,  // CPUID
    pub gcsr_prcfg1: usize, // PRCFG1
    pub gcsr_prcfg2: usize, // PRCFG2
    pub gcsr_prcfg3: usize, // PRCFG3

    // Saved Registers
    pub gcsr_save0: usize,  // SAVE0
    pub gcsr_save1: usize,  // SAVE1
    pub gcsr_save2: usize,  // SAVE2
    pub gcsr_save3: usize,  // SAVE3
    pub gcsr_save4: usize,  // SAVE4
    pub gcsr_save5: usize,  // SAVE5
    pub gcsr_save6: usize,  // SAVE6
    pub gcsr_save7: usize,  // SAVE7
    pub gcsr_save8: usize,  // SAVE8
    pub gcsr_save9: usize,  // SAVE9
    pub gcsr_save10: usize, // SAVE10
    pub gcsr_save11: usize, // SAVE11
    pub gcsr_save12: usize, // SAVE12
    pub gcsr_save13: usize, // SAVE13
    pub gcsr_save14: usize, // SAVE14
    pub gcsr_save15: usize, // SAVE15

    // Timer Registers
    pub gcsr_tid: usize,   // TID
    pub gcsr_tcfg: usize,  // TCFG
    pub gcsr_tval: usize,  // TVAL
    pub gcsr_cntc: usize,  // CNTC
    pub gcsr_ticlr: usize, // TICLR

    // Load Linked Buffers Registers
    pub gcsr_llbctl: usize, // LLBCTL

    // TLB Read Entry Registers
    pub gcsr_tlbrentry: usize, // TLBRENTRY
    pub gcsr_tlbrbadv: usize,  // TLBRBADV
    pub gcsr_tlbrera: usize,   // TLBRERA
    pub gcsr_tlbrsave: usize,  // TLBRSAVE
    pub gcsr_tlbrelo0: usize,  // TLBRELO0
    pub gcsr_tlbrelo1: usize,  // TLBRELO1
    pub gcsr_tlbrehi: usize,   // TLBREHI
    pub gcsr_tlbrprmd: usize,  // TLBRPRMD

    // Data Memory Write Registers
    pub gcsr_dmw0: usize, // DMW0
    pub gcsr_dmw1: usize, // DMW1
    pub gcsr_dmw2: usize, // DMW2
    pub gcsr_dmw3: usize, // DMW3

    // Pagetable address
    pub pgdl: usize,
    pub pgdh: usize,
}

macro_rules! gprs_getters {
  ($($reg_name:ident, $index:expr),*) => {
      $(
          pub fn $reg_name(&self) -> usize {
              self.x[$index]
          }
      )*
  }
}

macro_rules! gprs_setters {
  ($($set_name:ident, $index:expr),*) => {
      $(
          pub fn $set_name(&mut self, val: usize) {
              self.x[$index] = val;
          }
      )*
  }
}

impl LoongArch64ZoneContext {
    pub const fn new() -> LoongArch64ZoneContext {
        LoongArch64ZoneContext {
            x: [0; 32],
            sepc: 0,
            // 初始化 GCSR 寄存器
            gcsr_crmd: 0,
            gcsr_prmd: 0,
            gcsr_euen: 0,
            gcsr_misc: 0,
            gcsr_ectl: 0,
            gcsr_estat: 0,
            gcsr_era: 0,
            gcsr_badv: 0,
            gcsr_badi: 0,
            gcsr_eentry: 0,
            gcsr_tlbidx: 0,
            gcsr_tlbehi: 0,
            gcsr_tlbelo0: 0,
            gcsr_tlbelo1: 0,
            gcsr_asid: 0,
            gcsr_pgdl: 0,
            gcsr_pgdh: 0,
            gcsr_pgd: 0,
            gcsr_pwcl: 0,
            gcsr_pwch: 0,
            gcsr_stlbps: 0,
            gcsr_ravcfg: 0,
            gcsr_cpuid: 0,
            gcsr_prcfg1: 0,
            gcsr_prcfg2: 0,
            gcsr_prcfg3: 0,
            gcsr_save0: 0,
            gcsr_save1: 0,
            gcsr_save2: 0,
            gcsr_save3: 0,
            gcsr_save4: 0,
            gcsr_save5: 0,
            gcsr_save6: 0,
            gcsr_save7: 0,
            gcsr_save8: 0,
            gcsr_save9: 0,
            gcsr_save10: 0,
            gcsr_save11: 0,
            gcsr_save12: 0,
            gcsr_save13: 0,
            gcsr_save14: 0,
            gcsr_save15: 0,
            gcsr_tid: 0,
            gcsr_tcfg: 0,
            gcsr_tval: 0,
            gcsr_cntc: 0,
            gcsr_ticlr: 0,
            gcsr_llbctl: 0,
            gcsr_tlbrentry: 0,
            gcsr_tlbrbadv: 0,
            gcsr_tlbrera: 0,
            gcsr_tlbrsave: 0,
            gcsr_tlbrelo0: 0,
            gcsr_tlbrelo1: 0,
            gcsr_tlbrehi: 0,
            gcsr_tlbrprmd: 0,
            gcsr_dmw0: 0,
            gcsr_dmw1: 0,
            gcsr_dmw2: 0,
            gcsr_dmw3: 0,
            // pagetable of zone
            pgdl: 0,
            pgdh: 0,
        }
    }

    pub fn print_zone_context(&self) {
        info!("=============ZONE CONTEXT============");
        // get self addr in memory
        let self_addr = self as *const _ as usize;
        info!("self addr: {:#x}", self_addr);
        for (index, &register) in self.x.iter().enumerate() {
            info!("$r[{}]: {:#x}", index, register);
        }
        info!("sepc: {:#x}", self.sepc);
        info!("gcsr_crmd: {:#x}", self.gcsr_crmd);
        info!("gcsr_prmd: {:#x}", self.gcsr_prmd);
        info!("gcsr_euen: {:#x}", self.gcsr_euen);
        info!("gcsr_misc: {:#x}", self.gcsr_misc);
        info!("gcsr_ectl: {:#x}", self.gcsr_ectl);
        info!("gcsr_estat: {:#x}", self.gcsr_estat);
        info!("gcsr_era: {:#x}", self.gcsr_era);
        info!("gcsr_badv: {:#x}", self.gcsr_badv);
        info!("gcsr_badi: {:#x}", self.gcsr_badi);
        info!("gcsr_eentry: {:#x}", self.gcsr_eentry);
        info!("gcsr_tlbidx: {:#x}", self.gcsr_tlbidx);
        info!("gcsr_tlbehi: {:#x}", self.gcsr_tlbehi);
        info!("gcsr_tlbelo0: {:#x}", self.gcsr_tlbelo0);
        info!("gcsr_tlbelo1: {:#x}", self.gcsr_tlbelo1);
        info!("gcsr_asid: {:#x}", self.gcsr_asid);
        info!("gcsr_pgdl: {:#x}", self.gcsr_pgdl);
        info!("gcsr_pgdh: {:#x}", self.gcsr_pgdh);
        info!("gcsr_pgd: {:#x}", self.gcsr_pgd);
        info!("gcsr_pwcl: {:#x}", self.gcsr_pwcl);
        info!("gcsr_pwch: {:#x}", self.gcsr_pwch);
        info!("gcsr_stlbps: {:#x}", self.gcsr_stlbps);
        info!("gcsr_ravcfg: {:#x}", self.gcsr_ravcfg);
        info!("gcsr_cpuid: {:#x}", self.gcsr_cpuid);
        info!("gcsr_prcfg1: {:#x}", self.gcsr_prcfg1);
        info!("gcsr_prcfg2: {:#x}", self.gcsr_prcfg2);
        info!("gcsr_prcfg3: {:#x}", self.gcsr_prcfg3);
        info!("gcsr_save0: {:#x}", self.gcsr_save0);
        info!("gcsr_save1: {:#x}", self.gcsr_save1);
        info!("gcsr_save2: {:#x}", self.gcsr_save2);
        info!("gcsr_save3: {:#x}", self.gcsr_save3);
        info!("gcsr_save4: {:#x}", self.gcsr_save4);
        info!("gcsr_save5: {:#x}", self.gcsr_save5);
        info!("gcsr_save6: {:#x}", self.gcsr_save6);
        info!("gcsr_save7: {:#x}", self.gcsr_save7);
        info!("gcsr_save8: {:#x}", self.gcsr_save8);
        info!("gcsr_save9: {:#x}", self.gcsr_save9);
        info!("gcsr_save10: {:#x}", self.gcsr_save10);
        info!("gcsr_save11: {:#x}", self.gcsr_save11);
        info!("gcsr_save12: {:#x}", self.gcsr_save12);
        info!("gcsr_save13: {:#x}", self.gcsr_save13);
        info!("gcsr_save14: {:#x}", self.gcsr_save14);
        info!("gcsr_save15: {:#x}", self.gcsr_save15);
        info!("gcsr_tid: {:#x}", self.gcsr_tid);
        info!("gcsr_tcfg: {:#x}", self.gcsr_tcfg);
        info!("gcsr_tval: {:#x}", self.gcsr_tval);
        info!("gcsr_cntc: {:#x}", self.gcsr_cntc);
        info!("gcsr_ticlr: {:#x}", self.gcsr_ticlr);
        info!("gcsr_llbctl: {:#x}", self.gcsr_llbctl);
        info!("gcsr_tlbrentry: {:#x}", self.gcsr_tlbrentry);
        info!("gcsr_tlbrbadv: {:#x}", self.gcsr_tlbrbadv);
        info!("gcsr_tlbrera: {:#x}", self.gcsr_tlbrera);
        info!("gcsr_tlbrsave: {:#x}", self.gcsr_tlbrsave);
        info!("gcsr_tlbrelo0: {:#x}", self.gcsr_tlbrelo0);
        info!("gcsr_tlbrelo1: {:#x}", self.gcsr_tlbrelo1);
        info!("gcsr_tlbrehi: {:#x}", self.gcsr_tlbrehi);
        info!("gcsr_tlbrprmd: {:#x}", self.gcsr_tlbrprmd);
        info!("gcsr_dmw0: {:#x}", self.gcsr_dmw0);
        info!("gcsr_dmw1: {:#x}", self.gcsr_dmw1);
        info!("gcsr_dmw2: {:#x}", self.gcsr_dmw2);
        info!("gcsr_dmw3: {:#x}", self.gcsr_dmw3);
        info!("pgdl: {:#x}", self.pgdl);
        info!("pgdh: {:#x}", self.pgdh);
    }

    gprs_getters!(
        get_ra, 1, get_a0, 4, get_a1, 5, get_a2, 6, get_a3, 7, get_a4, 8, get_a5, 9, get_a6, 10,
        get_a7, 11
    );
    gprs_setters!(set_a0, 4);
}
