use crate::device::common::MMIODerefWrapper;
use core::arch::asm;
use tock_registers::fields::FieldValue;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    debug!(
        "loongarch64: arch_send_event: sending event to cpu: {}, sgi_num: {}",
        cpu_id, sgi_num
    );
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("loongarch64: arch_send_event: invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    let mut val: u32 = 1 << 31;
    val |= sgi_num as u32;
    val |= (cpu_id as u32) << 16;
    unsafe {
        asm!("iocsrwr.w {}, {}", in(reg) val, in(reg) 0x1040);
    }
}

register_bitfields! [
  u32,
  pub IpiStatus [ IPISTATUS OFFSET(0) NUMBITS(32) ],
  pub IpiEnable [ IPIENABLE OFFSET(0) NUMBITS(32) ],
  pub IpiSet [ IPISET OFFSET(0) NUMBITS(32) ],
  pub IpiClear [ IPICLEAR OFFSET(0) NUMBITS(32) ],
];

register_bitfields! [
  u64,
  pub Mailbox0 [ MAILBOX0 OFFSET(0) NUMBITS(64) ],
  pub Mailbox1 [ MAILBOX1 OFFSET(0) NUMBITS(64) ],
  pub Mailbox2 [ MAILBOX2 OFFSET(0) NUMBITS(64) ],
  pub Mailbox3 [ MAILBOX3 OFFSET(0) NUMBITS(64) ],
];

register_structs! {
  #[allow(non_snake_case)]
  pub IpiRegisters {
    (0x00 => pub ipi_status: ReadOnly<u32, IpiStatus::Register>),
    (0x04 => pub ipi_enable: ReadWrite<u32, IpiEnable::Register>),
    (0x08 => pub ipi_set: WriteOnly<u32, IpiSet::Register>),
    (0x0c => pub ipi_clear: WriteOnly<u32, IpiClear::Register>),
    (0x10 => _reserved0: [u8; 0x10]),
    (0x20 => pub mailbox0: ReadWrite<u64, Mailbox0::Register>),
    (0x28 => pub mailbox1: ReadWrite<u64, Mailbox1::Register>),
    (0x30 => pub mailbox2: ReadWrite<u64, Mailbox2::Register>),
    (0x38 => pub mailbox3: ReadWrite<u64, Mailbox3::Register>),
    (0x40 => @END),
  }
}

const MMIO_BASE: usize = 0x8000_0000_1fe0_0000;
const IPI_MMIO_BASE: usize = MMIO_BASE;

pub static CORE0_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE + 0x1000) };
pub static CORE1_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE + 0x1100) };
pub static CORE2_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE + 0x1200) };
pub static CORE3_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE + 0x1300) };

// ipi actions
pub const SMP_BOOT_CPU: usize = 0x1;
pub const SMP_RESCHEDULE: usize = 0x2;
pub const SMP_CALL_FUNCTION: usize = 0x4;
// customized actions :), since there is no docs on this yet
pub const HVISOR_START_VCPU: usize = 0x8;

fn iocsr_mbuf_send_box_lo(a: usize) -> usize {
    a << 1
}
fn iocsr_mbuf_send_box_hi(a: usize) -> usize {
    (a << 1) + 1
}

// allow unused for now
#[allow(unused_assignments)]
pub fn mail_send(data: usize, cpu_id: usize, mailbox_id: usize) {
    // the high and low 32 bits should be sent separately
    // first high 32 bits, then low 32 bits
    let mut high = data >> 32;
    let mut low = data & 0xffffffff;
    let mut val: usize = 0;
    // send high 32 bits
    val = 1 << 31;
    val |= iocsr_mbuf_send_box_hi(mailbox_id) << 2;
    val |= cpu_id << 16;
    val |= high << 32;
    // info!("(mail_send) sending high 32 bits, actual packed value: {:#x}", val);
    unsafe {
        asm!("iocsrwr.d {}, {}", in(reg) val, in(reg) 0x1048);
    }
    // send low 32 bits
    val = 1 << 31;
    val |= iocsr_mbuf_send_box_lo(mailbox_id) << 2;
    val |= cpu_id << 16;
    val |= low << 32;
    // info!("(mail_send) sending low 32 bits, actual packed value: {:#x}", val);
    unsafe {
        asm!("iocsrwr.d {}, {}", in(reg) val, in(reg) 0x1048);
    }
}

fn ffs(a: usize) -> usize {
    // find first set bit, least significant bit is at position 1
    // if a is 0, return 0
    if a == 0 {
        return 0;
    }
    let mut a = a;
    let mut i = 0;
    while (a & 1) == 0 {
        a >>= 1;
        i += 1;
    }
    i + 1
}

#[allow(unused_assignments)]
pub fn ipi_write_action(cpu_id: usize, action: usize) {
    let mut irq: u32 = 0;
    let mut action = action;
    loop {
        irq = ffs(action) as u32;
        if irq == 0 {
            break;
        }
        let mut val: u32 = 1 << 31;
        val |= irq - 1;
        val |= (cpu_id as u32) << 16;
        // info!("(ipi_write_action) sending action: {:#x}", val);
        unsafe {
            asm!("iocsrwr.w {}, {}", in(reg) val, in(reg) 0x1040);
        }
        action &= !(1 << (irq - 1));
    }
}

pub fn clear_all_ipi(cpu_id: usize) {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("(clear_all_ipi) invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    ipi.ipi_clear.write(IpiClear::IPICLEAR.val(0xffffffff));
    info!(
        "(clear_all_ipi) IPI status for cpu {}: {:#x}",
        cpu_id,
        ipi.ipi_status.read(IpiStatus::IPISTATUS)
    );
}

pub fn get_ipi_status(cpu_id: usize) -> u32 {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("(get_ipi_status) invalid cpu_id: {}", cpu_id);
            return 0;
        }
    };
    ipi.ipi_status.read(IpiStatus::IPISTATUS)
}
