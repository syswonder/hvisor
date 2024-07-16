use super::ipi::*;
use super::zone::ZoneContext;
use crate::device::common::MMIODerefWrapper;
use crate::percpu::this_cpu_data;
use core::arch::asm;
use core::fmt::{self, Debug, Formatter};
use loongArch64::register::cpuid;
use loongArch64::register::pgdl;
use tock_registers::interfaces::Writeable;

use crate::{
    consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::VirtAddr,
};

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub ctx: ZoneContext,
    pub stack_top: usize,
    pub cpuid: usize,
    pub power_on: bool,
    pub init: bool,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let ret = ArchCpu {
            ctx: super::trap::dump_reset_gcsrs(),
            stack_top: 0,
            cpuid,
            power_on: false,
            init: false,
        };
        ret
    }
    pub fn get_cpuid(&self) -> usize {
        self.cpuid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_cpuid() + 1) as usize * PER_CPU_SIZE
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) {
        self.ctx.sepc = entry;
        self.stack_top = self.stack_top() as usize;
    }
    pub fn run(&mut self) -> ! {
        assert!(this_cpu_id() == self.get_cpuid());
        this_cpu_data().activate_gpm();
        self.power_on = true;
        if !self.init {
            self.init(
                this_cpu_data().cpu_on_entry,
                this_cpu_data().id,
                this_cpu_data().opaque,
            );
            self.init = true;
        }
        info!(
            "loongarch64: CPU{} run@{:#x}",
            self.get_cpuid(),
            self.ctx.sepc
        );
        info!("loongarch64: @{:#x?}", self);
        // step 1: enable guest mode
        // step 2: set guest entry to era
        // step 3: run ertn and enter guest mode
        let ctx_addr = &mut self.ctx as *mut ZoneContext;
        debug!(
            "loongarch64: ArchCpu::run: percpu_s={:#x}",
            self.stack_top() - PER_CPU_SIZE
        );
        debug!(
            "loongarch64: ArchCpu::run: ctx_addr={:#x}, size={}",
            ctx_addr as usize,
            core::mem::size_of::<ZoneContext>()
        );
        debug!(
            "loongarch64: ArchCpu::run: stack_tp={:#x}",
            self.stack_top()
        );

        unsafe {
            asm!(
                "csrwr {}, {LOONGARCH_CSR_SAVE3}",
                "csrwr {}, {LOONGARCH_CSR_SAVE4}",
                in(reg) (ctx_addr as usize + core::mem::size_of::<ZoneContext>()),
                in(reg) self.stack_top(),
                LOONGARCH_CSR_SAVE3 = const 0x33,
                LOONGARCH_CSR_SAVE4 = const 0x34,
            );
        }

        super::trap::_vcpu_return(ctx_addr as usize);

        panic!("loongarch64: ArchCpu::run: unreachable");
    }
    pub fn idle(&self) -> ! {
        info!("loongarch64: ArchCpu::idle: cpuid={}", self.get_cpuid());
        loop {}
    }
}

pub fn this_cpu_id() -> usize {
    cpuid::read().core_id()
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    let start_addr = start_addr & 0x0000_ffff_ffff_ffff;
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpuid {
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            panic!("loongarch64: cpu_start: invalid cpuid={}", cpuid);
        }
    };
    ipi.ipi_enable.write(IpiEnable::IPIENABLE.val(0xffffffff));
    let entry_addr = start_addr;
    mail_send(entry_addr, cpuid, 0);
    ipi_write_action(cpuid, SMP_BOOT_CPU);
}
