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
//      Yulong Han <wheatfox17@icloud.com>
//
use crate::arch::cpu::this_cpu_id;
use crate::consts::IPI_EVENT_CLEAR_INJECT_IRQ;
use crate::device::common::MMIODerefWrapper;
use core::arch::asm;
use core::ptr::write_volatile;
use loongArch64::cpu;
use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::*;
use loongArch64::time;
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
    // just call ipi_write_action
    ipi_write_action(cpu_id as usize, sgi_num as usize);
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
const IPI_ANY_SEND_BASE: usize = MMIO_BASE + 0x1158;

// IPI registers, use this if you don't want to use the percore-IPI feature
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
pub fn mail_send_percore(data: usize, cpu_id: usize, mailbox_id: usize) {
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
    // debug!("(mail_send) sending high 32 bits, actual packed value: {:#x}", val);
    unsafe {
        // asm!("iocsrwr.d {}, {}", in(reg) val, in(reg) 0x1048);
        write_volatile(IPI_MMIO_MAIL_SEND as *mut u64, val as u64);
    }
    // send low 32 bits
    val = 1 << 31;
    val |= iocsr_mbuf_send_box_lo(mailbox_id) << 2;
    val |= cpu_id << 16;
    val |= low << 32;
    // debug!("(mail_send) sending low 32 bits, actual packed value: {:#x}", val);
    unsafe {
        // asm!("iocsrwr.d {}, {}", in(reg) val, in(reg) 0x1048);
        write_volatile(IPI_MMIO_MAIL_SEND as *mut u64, val as u64);
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

const IPI_MMIO_IPI_SEND: usize = MMIO_BASE + 0x1040; // 32 bits Write Only
const IPI_MMIO_MAIL_SEND: usize = MMIO_BASE + 0x1048; // 64 bits Write Only

#[allow(unused_assignments)]
pub fn ipi_write_action_percore(cpu_id: usize, _action: usize) {
    let mut irq: u32 = 0;
    let mut action = _action;
    debug!(
        "loongarch64::ipi_write_action sending action: {:#x} to cpu: {}",
        action, cpu_id
    );
    loop {
        irq = ffs(action) as u32;
        if irq == 0 {
            break;
        }
        let mut val: u32 = 1 << 31;
        val |= irq - 1;
        val |= (cpu_id as u32) << 16;
        debug!(
            "loongarch64::ipi_write_action writing value {:#x} to MMIO address: {:#x}",
            val, IPI_MMIO_IPI_SEND
        );
        unsafe {
            //     asm!("iocsrwr.w {}, {}", in(reg) val, in(reg) 0x1040);
            write_volatile(IPI_MMIO_IPI_SEND as *mut u32, val);
        }
        debug!(
            "loongarch64::ipi_write_action sent irq: {} to cpu: {} !",
            irq, cpu_id
        );
        action &= !(1 << (irq - 1));
    }
    debug!(
        "loongarch64::ipi_write_action finished sending to cpu: {}",
        cpu_id
    );
}

pub fn ipi_write_action(cpu_id: usize, _action: usize) {
    // just write _action directly to the target cpu legacy IPI registers
    // which is the IPI_SET register
    debug!(
        "ipi_write_action_legacy: sending action: {:#x} to cpu: {}",
        _action, cpu_id
    );
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("ipi_write_action_legacy: invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    ipi.ipi_set.write(IpiSet::IPISET.val(_action as u32));
    debug!(
        "ipi_write_action_legacy: finished sending action: {:#x} to cpu: {}",
        _action, cpu_id
    );
}

pub fn mail_send(data: usize, cpu_id: usize, mailbox_id: usize) {
    // just write data to the target cpu mailbox registers
    // which is the mailbox0 register
    debug!(
        "mail_send: sending data: {:#x} to cpu: {}, mailbox_id: {}",
        data, cpu_id, mailbox_id
    );
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("mail_send: invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    match mailbox_id {
        0 => ipi.mailbox0.write(Mailbox0::MAILBOX0.val(data as u64)),
        1 => ipi.mailbox1.write(Mailbox1::MAILBOX1.val(data as u64)),
        2 => ipi.mailbox2.write(Mailbox2::MAILBOX2.val(data as u64)),
        3 => ipi.mailbox3.write(Mailbox3::MAILBOX3.val(data as u64)),
        _ => {
            error!("mail_send: invalid mailbox_id: {}", mailbox_id);
            return;
        }
    }
    debug!(
        "mail_send: finished sending data: {:#x} to cpu: {}, mailbox_id: {}",
        data, cpu_id, mailbox_id
    );
}

pub fn enable_ipi(cpu_id: usize) {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("enable_ipi: invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    ipi.ipi_enable.write(IpiEnable::IPIENABLE.val(0xffffffff));
    debug!("enable_ipi: IPI enabled for cpu {}", cpu_id);
}

pub fn clear_all_ipi(cpu_id: usize) {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("clear_all_ipi: invalid cpu_id: {}", cpu_id);
            return;
        }
    };
    ipi.ipi_clear.write(IpiClear::IPICLEAR.val(0xffffffff));
    debug!(
        "clear_all_ipi: IPI status for cpu {}: {:#x}",
        cpu_id,
        ipi.ipi_status.read(IpiStatus::IPISTATUS)
    );
}

pub fn reset_ipi(cpu_id: usize) {
    // clear all IPIs and enable all IPIs
    clear_all_ipi(cpu_id);
    enable_ipi(cpu_id);
}

pub fn get_ipi_status(cpu_id: usize) -> u32 {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("get_ipi_status: invalid cpu_id: {}", cpu_id);
            return 0;
        }
    };
    ipi.ipi_status.read(IpiStatus::IPISTATUS)
}

pub fn ecfg_ipi_enable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::IPI;
    ecfg::set_lie(lie_);
    info!(
        "ecfg ipi enabled on cpu {}, current lie: {:?}",
        this_cpu_id(),
        lie_
    );
}

pub fn ecfg_ipi_disable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ & !LineBasedInterrupt::IPI;
    ecfg::set_lie(lie_);
    info!(
        "ecfg ipi disabled on cpu {}, current lie: {:?}",
        this_cpu_id(),
        lie_
    );
}

pub fn dump_ipi_registers() {
    info!(
        "dump_ipi_registers: dumping IPI registers for this cpu {}",
        this_cpu_id()
    );
    let ipi: &MMIODerefWrapper<IpiRegisters> = match this_cpu_id() {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        _ => {
            error!("dump_ipi_registers: invalid cpu_id: {}", this_cpu_id());
            return;
        }
    };
    println!(
        "ipi_status: {:#x}, ipi_enable: {:#x}",
        ipi.ipi_status.read(IpiStatus::IPISTATUS),
        ipi.ipi_enable.read(IpiEnable::IPIENABLE),
    );
    println!(
        "mailbox0: {:#x}, mailbox1: {:#x}, mailbox2: {:#x}, mailbox3: {:#x}",
        ipi.mailbox0.read(Mailbox0::MAILBOX0),
        ipi.mailbox1.read(Mailbox1::MAILBOX1),
        ipi.mailbox2.read(Mailbox2::MAILBOX2),
        ipi.mailbox3.read(Mailbox3::MAILBOX3)
    );
}

pub fn arch_check_events(event: Option<usize>) {
    match event {
        Some(IPI_EVENT_CLEAR_INJECT_IRQ) => {
            // clear the injected IPI interrupt
            use crate::device::irqchip::ls7a2000::clear_hwi_injected_irq;
            clear_hwi_injected_irq();
        }
        _ => {
            panic!("arch_check_events: unhandled event: {:?}", event);
        }
    }
}

pub fn arch_prepare_send_event() {
    use crate::event::fetch_event;
    while !fetch_event(cpu_id).is_none() {}
    debug!(
        "loongarch64:: send_event: cpu_id: {}, ipi_int_id: {}, event_id: {}",
        cpu_id, ipi_int_id, event_id
    );
}
