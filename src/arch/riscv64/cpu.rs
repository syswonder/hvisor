use super::csr::*;
use crate::{
    consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::{PhysAddr, VirtAddr}, 
};

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub x: [usize; 32], //x0~x31
    pub hstatus: usize,
    pub sstatus: usize,
    pub sepc: usize,
    pub stack_top: usize,
    pub hartid: usize,
}

impl ArchCpu {
    pub fn new(hartid: usize) -> Self {
        ArchCpu {
            x: [0; 32],
            hstatus: 0,
            sstatus: 0,
            sepc: 0,
            stack_top: 0,
            hartid,
        }
    }
    pub fn get_hartid(&self) -> usize {
        self.hartid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_hartid() + 1) as usize * PER_CPU_SIZE - 8
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) {
        //self.sepc = guest_test as usize as u64;
        write_csr!(CSR_SSCRATCH, self as *const _ as usize); //arch cpu pointer
        self.sepc = entry;
        self.hstatus = 1 << 7 | 2 << 32; //HSTATUS_SPV | HSTATUS_VSXL_64
        self.sstatus = 1 << 8 | 1 << 63 | 3 << 13 | 3 << 15; //SPP
        self.stack_top = self.stack_top() as usize;
        self.x[10] = cpu_id; //cpu id
        self.x[11] = dtb; //dtb addr
        trace!("stack_top: {:#x}", self.stack_top);

        // write_csr!(CSR_SSTATUS, self.sstatus);
        // write_csr!(CSR_HSTATUS, self.hstatus);
        // write_csr!(CSR_SEPC, self.sepc);
        set_csr!(CSR_HIDELEG, 1 << 2 | 1 << 6 | 1 << 10); //HIDELEG_VSSI | HIDELEG_VSTI | HIDELEG_VSEI
        set_csr!(CSR_HEDELEG, 1 << 8 | 1 << 12 | 1 << 13 | 1 << 15); //HEDELEG_ECU | HEDELEG_IPF | HEDELEG_LPF | HEDELEG_SPF
        set_csr!(CSR_HCOUNTEREN, 1 << 1); //HCOUNTEREN_TM
                                          //In VU-mode, a counter is not readable unless the applicable bits are set in both hcounteren and scounteren.
        set_csr!(CSR_SCOUNTEREN, 1 << 1);
        write_csr!(CSR_HTIMEDELTA, 0);
        set_csr!(CSR_HENVCFG, 1 << 63);
        //write_csr!(CSR_VSSTATUS, 1 << 63 | 3 << 13 | 3 << 15); //SSTATUS_SD | SSTATUS_FS_DIRTY | SSTATUS_XS_DIRTY

        // enable all interupts
        set_csr!(CSR_SIE, 1 << 9 | 1 << 5 | 1 << 1); //SEIE STIE SSIE
                                                     // write_csr!(CSR_HIE, 1 << 12 | 1 << 10 | 1 << 6 | 1 << 2); //SGEIE VSEIE VSTIE VSSIE
        write_csr!(CSR_HIE, 0);
        write_csr!(CSR_VSTVEC, 0);
        write_csr!(CSR_VSSCRATCH, 0);
        write_csr!(CSR_VSEPC, 0);
        write_csr!(CSR_VSCAUSE, 0);
        write_csr!(CSR_VSTVAL, 0);
        write_csr!(CSR_HVIP, 0);
        write_csr!(CSR_VSATP, 0);
        let mut value: usize;
        value = read_csr!(CSR_SEPC);
        info!("CSR_SEPC: {:#x}", value);
        value = read_csr!(CSR_STVEC);
        info!("CSR_STVEC: {:#x}", value);
        value = read_csr!(CSR_VSATP);
        info!("CSR_VSATP: {:#x}", value);
        value = read_csr!(CSR_HGATP);
        info!("CSR_HGATP: {:#x}", value);
        //unreachable!();
    }
    pub fn run(&mut self) {
        extern "C" {
            fn vcpu_arch_entry();
        }
        unsafe {
            vcpu_arch_entry();
        }
    }

    pub fn idle(&self) {
        unsafe {
            core::arch::asm!("wfi");
        }
        println!("CPU{} weakup!", self.hartid);
        debug!("sip: {:#x}", read_csr!(CSR_SIP));
        clear_csr!(CSR_SIP, 1 << 1);
        debug!("sip*: {:#x}", read_csr!(CSR_SIP));
    }
}

fn this_cpu_arch() -> &'static mut ArchCpu {
    let sscratch = read_csr!(CSR_SSCRATCH);
    if sscratch == 0 {
        panic!("CSR_SSCRATCH unintialized!");
    }
    unsafe { &mut *(sscratch as *mut ArchCpu) }
}

pub fn this_cpu_id() -> usize {
    this_cpu_arch().get_hartid()
}

const HV_BASE: VirtAddr = 0x80200000;
const HV_PHY_BASE: PhysAddr = 0x80200000;

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize)  {
    if let Some(e) = sbi_rt::hart_start(cpuid, HV_PHY_BASE, opaque).err() {
        panic!("cpu_start error: {:#x?}", e);
    }
}