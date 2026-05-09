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
//      Ming Shen  <boneinscri@163.com>
//
use crate::arch::cpu::this_cpu_id;
use crate::consts::{IPI_EVENT_CLEAR_INJECT_IRQ, MAX_CPU_NUM};
use crate::event::{IPI_EVENT_VIRTIO_INJECT_IRQ, IPI_EVENT_WAKEUP, IPI_EVENT_SHUTDOWN, IPI_EVENT_WAKEUP_VIRTIO_DEVICE};
use crate::cpu_data::{get_cpu_data, this_zone};
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

    if sgi_num == IPI_EVENT_WAKEUP as u64 {
        ipi_send_general(cpu_id as usize, SMP_BOOT_CPU as u32);
    } else if sgi_num == IPI_EVENT_SHUTDOWN as u64 {
        ipi_send_general(cpu_id as usize, HVISOR_SHUTDOWN as u32);
    } else if sgi_num == IPI_EVENT_VIRTIO_INJECT_IRQ as u64 {
        ipi_send_general(cpu_id as usize, HVISOR_EVENT_VIRTIO_INJECT_IRQ as u32);
    } else if sgi_num == IPI_EVENT_WAKEUP_VIRTIO_DEVICE as u64  {
        ipi_send_general(cpu_id as usize, HVISOR_EVENT_WAKEUP_VIRTIO_DEVICE as u32);
    } else if sgi_num == IPI_EVENT_CLEAR_INJECT_IRQ as u64 {
        ipi_send_general(cpu_id as usize, HVISOR_EVENT_VIRTIO_CLEAR_IRQ as u32);    
    } else {
        panic!("unknown sgi_num, {}", sgi_num);
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

// boneinscri -- 2026.04 (for 3a6000 smp)
const MMIO_BASE_2: usize = 0x8000_0000_1fe1_0000;
const IPI_MMIO_BASE_2: usize = MMIO_BASE_2;
pub static CORE4_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE_2 + 0x1000) };
pub static CORE5_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE_2 + 0x1100) };
pub static CORE6_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE_2 + 0x1200) };
pub static CORE7_IPI: MMIODerefWrapper<IpiRegisters> =
    unsafe { MMIODerefWrapper::new(IPI_MMIO_BASE_2 + 0x1300) };


// ipi actions
pub const SMP_BOOT_CPU: usize = 0x1;
pub const SMP_RESCHEDULE: usize = 0x2;
pub const SMP_CALL_FUNCTION: usize = 0x4;
// customized actions :), since there is no docs on this yet
pub const HVISOR_START_VCPU: usize = 0x8;

// boneinscri 2026.04
pub const HVISOR_SHUTDOWN: usize = 0x40;
pub const HVISOR_EVENT_VIRTIO_INJECT_IRQ: usize = 0x80;
pub const HVISOR_EVENT_WAKEUP_VIRTIO_DEVICE: usize = 0x100;
pub const HVISOR_EVENT_VIRTIO_CLEAR_IRQ: usize = 0x200;

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

        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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

        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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

        
        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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

        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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
    // enable_ipi(cpu_id);
}

pub fn get_ipi_status(cpu_id: usize) -> u32 {
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpu_id {
        0 => &CORE0_IPI,
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,

        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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
    // info!(
    //     "ecfg ipi enabled on cpu {}, current lie: {:?}",
    //     this_cpu_id(),
    //     lie_
    // );
}

pub fn ecfg_ipi_disable() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ & !LineBasedInterrupt::IPI;
    ecfg::set_lie(lie_);
    // info!(
    //     "ecfg ipi disabled on cpu {}, current lie: {:?}",
    //     this_cpu_id(),
    //     lie_
    // );
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

        // boneinscri 2026.04 (3a6000 smp)
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
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

pub fn arch_prepare_send_event(cpu_id: usize, ipi_int_id: usize, event_id: usize) {
    use crate::event::fetch_event;
    while !fetch_event(cpu_id).is_none() {}
    debug!(
        "loongarch64:: send_event: cpu_id: {}, ipi_int_id: {}, event_id: {}",
        cpu_id, ipi_int_id, event_id
    );
}



// IPI state per cpu (ref to kvm)
// boneinscri --2026.04
pub const IOCSR_IPI_BASE: usize = 0x1000;
pub const IOCSR_IPI_STATUS: usize = 0x000;
pub const INT_IPI: usize = 12;
pub const IOCSR_IPI_EN: usize = 0x004;
pub const IOCSR_IPI_SET: usize = 0x008;
pub const IOCSR_IPI_CLEAR: usize = 0x00c;
pub const IOCSR_IPI_BUF_20: usize = 0x020;
pub const IOCSR_IPI_BUF_28: usize = 0x028;
pub const IOCSR_IPI_BUF_30: usize = 0x030;
pub const IOCSR_IPI_BUF_38: usize = 0x038;
pub const IOCSR_IPI_SEND: usize = 0x040;
pub const IOCSR_MAIL_SEND: usize = 0x048;
pub const IOCSR_ANY_SEND: usize = 0x158;
pub const IOCSR_IPI_BUF_END: usize = IOCSR_IPI_BUF_38 + 7;


#[derive(Debug)]
pub struct LoongArch64IpiState {
    pub status: u32,
    pub en: u32,
    pub set: u32,
    pub clear: u32,
    pub buf: [u64; MAX_CPU_NUM],
}

impl LoongArch64IpiState {
    pub fn new() -> Self {
        Self {
            status: 0,
            en: 0,
            set: 0,
            clear: 0,
            buf: [0; MAX_CPU_NUM],
        }
    }
}

pub fn write_mailbox(pcpu_id: usize, offset: usize, len: usize, val: usize) {
    let pcpu = get_cpu_data(pcpu_id);
    
    if offset < 0x20 {
        panic!("ipi read mailbox, offset = {:#x}, len = {:#x}", offset, len);
    }
    
    let buf_offset = (offset - 0x20) as usize;
    let idx = buf_offset / 8;
    
    let mut ipistate = pcpu.arch_cpu.ipi_state.lock();
    let pbuf = &mut ipistate.buf[idx];

    match len {
        1 => {
            let byte_ptr = pbuf as *mut u64 as *mut u8;
            unsafe { *byte_ptr = val as u8 };
        },
        2 => {
            let short_ptr = pbuf as *mut u64 as *mut u16;
            unsafe { *short_ptr = val as u16 };
        },
        4 => {
            let int_ptr = pbuf as *mut u64 as *mut u32;
            unsafe { *int_ptr = val as u32 };
        },
        8 => {
            *pbuf = val as u64;
        },
        _ => {
            warn!("write_mailbox, invalid length {:#x}", len);
        }
    }
}

pub fn ipi_clear(pcpu_id: usize, data: usize) { 
    let pcpu = get_cpu_data(pcpu_id);
    let mut ipistate = pcpu.arch_cpu.ipi_state.lock();
    ipistate.status &= !(data as u32);
    let status = ipistate.status; 
    drop(ipistate);

    if status == 0 {   
        let cur_pcpu_id = this_cpu_id();
        if cur_pcpu_id != pcpu_id {
            panic!("ipi_clear, need to support vcpu");
        } else {
            pcpu.arch_cpu.remove_irq(INT_IPI);
        }
        // TODO : for vcpu , inject IPI interrupt
    }
}

// TODO: modify to vcpu
pub fn get_target_cpu_id(data: usize) -> usize {
    let target_cpu_id = ((data & 0xffffffff) >> 16) & 0x3ff;
    if target_cpu_id < MAX_CPU_NUM {
        let zone = this_zone();
        let cpu_set = zone.read().cpu_set();
        let result = cpu_set.iter().nth(target_cpu_id);
        drop(cpu_set);
        match result {
            Some(id) => id,
            None => {
                warn!("get_target_cpu_id, invalid target cpu id {:#x}, ignore", target_cpu_id);
                usize::MAX
            }
        }
    } else {
        panic!("invalid target cpu id {:#x}", target_cpu_id);
    }
}

pub fn ipi_send_general(target_cpu_id: usize, action: u32) {
    let target_cpu = get_cpu_data(target_cpu_id);
    let mut ipistate = target_cpu.arch_cpu.ipi_state.lock();
    let status = ipistate.status;
    ipistate.status |= action;    

    if (status == 0) {
        // TODO : for vcpu , inject IPI interrupt
        // pay attention to the target_cpu_id, it should be the real cpu id, not the vcpu id
        ipi_write_action(target_cpu_id as usize, action as usize);
    }
}

pub fn ipi_send(data: usize) {
    let target_cpu_id = get_target_cpu_id(data);
    if target_cpu_id == usize::MAX {
        return;
    }
    let action = (1usize << (data & 0x1f)) as u32;
    ipi_send_general(target_cpu_id, action);
}

pub fn send_ipi_data(target_cpu_id: usize, addr: usize, data: usize) {
    let mut mask = 0;
    let mut val = 0;

    if (data >> 27) & 0xf != 0 {
        val = loongarch_ipi_readl(target_cpu_id, addr, 4);
        if val == usize::MAX {
            panic!("send_ipi_data, read data from addr {:#x} failed", addr);
        }
        for i in 0..4 {
            if (data & (1usize << (27 + i))) != 0 {
                mask |= 0xff << (i * 8);
            }
        }
        val &= mask;
    }

    val |= (data >> 32) & !mask;
    let ret = loongarch_ipi_writel(target_cpu_id, addr, val, 4);
    if ret != val {
        panic!("send_ipi_data, write data to addr {:#x} failed", addr);
    }
}

pub fn any_send(data: usize) {
    let cpu = ((data & 0xffffffff) >> 16) & 0x3ff;
    if cpu != 0 && cpu != 4 {
        error!("cpu_id = {}", cpu);
        panic!("any_send 1, check it carefullly");
    }
    let target_cpu_id = get_target_cpu_id(data);
    if target_cpu_id == usize::MAX {
        return;
    }
    if target_cpu_id != 0 && target_cpu_id != 4 {
        error!("cpu_id = {}", cpu);
        panic!("any_send 2, check it carefullly");
    }
    let offset = data & 0xffff;
    warn!("[Look this] any_send, offset {:#x}, data {:#x}", offset, data);
    send_ipi_data(target_cpu_id, offset, data);
}

pub fn mail_send_iocsr(data: usize) {
    let target_cpu_id = get_target_cpu_id(data);
    if target_cpu_id == usize::MAX {
        return;
    }
    let mailbox = ((data & 0xffffffff) >> 2) & 0x7;
    let offset = IOCSR_IPI_BASE + IOCSR_IPI_BUF_20 + mailbox * 4;
    warn!("[Look this]mail_send_iocsr, offset {:#x}, data {:#x}", offset, data);
    send_ipi_data(target_cpu_id, offset, data);
}

pub fn read_mailbox(pcpu_id: usize, offset: usize, len: usize) -> usize {
    let res = 0;
    let pcpu = get_cpu_data(pcpu_id);
    let ipi_state = &pcpu.arch_cpu.ipi_state;
    if offset < 0x20 {
        panic!("ipi read mailbox, offset = {:#x}, len = {:#x}\n", offset, len);
    }
    let idx: usize = ((offset - 0x20) / 8).try_into().unwrap();

    let ipistate = ipi_state.lock();
    let data = ipistate.buf[idx];    
    
    match len {
        1 => data & 0xff,
        2 => data & 0xffff,
        4 => data & 0xffffffff,
        8 => data,
        _ => {
            panic!("read_mailbox: unknown data len: {}", len);
        }
    };
    res    
}

// TODO : add vcpu for loongarch_ipi_readl and loongarch_ipi_writel
pub fn loongarch_ipi_readl(pcpu_id: usize, addr: usize, len: usize) -> usize {
    let offset = (addr & 0x1ff);
    if offset & (len - 1) != 0 {
        warn!("Unaligned access");
    }
    let mut res = 0;
    
    match offset {
        IOCSR_IPI_STATUS => {
            // this overhead is high
            let pcpu = get_cpu_data(pcpu_id);
            let ipistate = pcpu.arch_cpu.ipi_state.lock();
            res = ipistate.status as usize;
        } 
        IOCSR_IPI_EN => {
            let pcpu = get_cpu_data(pcpu_id);
            let ipistate = pcpu.arch_cpu.ipi_state.lock();
            res = ipistate.en as usize;
        }
        IOCSR_IPI_SET => {
            res = 0;
        }
        IOCSR_IPI_CLEAR => {
            res = 0;
        }
        IOCSR_IPI_BUF_20..=IOCSR_IPI_BUF_END => {
            if offset + len > IOCSR_IPI_BUF_38 + 8 {
                panic!("ipi readl IOCSR_IPI_BUF, offset = {:#x}, len = {:#x}", offset, len);    
            }
            res = read_mailbox(pcpu_id, offset, len);
        }
        _ => {
            panic!("Invalid IPI read offset: {:#x}", offset);
        }
    }

    res
}

pub fn loongarch_ipi_writel(pcpu_id: usize, addr: usize, val: usize, len: usize) -> usize {
    let mut res = val;
    let offset = (addr & 0x1ff);
    if offset & (len - 1) != 0 {
        warn!("Unaligned access");
    }

    match offset {
        IOCSR_IPI_SEND => {
            // overhead is high
            ipi_send(val);
        }
        IOCSR_IPI_CLEAR => {
            // overhead is high
            ipi_clear(pcpu_id, val);
        }
        IOCSR_IPI_STATUS => {
            panic!("ipi writel IOCSR_IPI_STATUS, pcpu_id = {}, val = {:#x}", pcpu_id, val);
        }
        IOCSR_IPI_EN => {
            let mut pcpu = get_cpu_data(pcpu_id);
            let mut ipistate = pcpu.arch_cpu.ipi_state.lock();
            ipistate.en = val as u32;
        }
        IOCSR_IPI_SET => {
            panic!("ipi writel IOCSR_IPI_SET, pcpu_id = {}, val = {:#x}", pcpu_id, val);
        }
        IOCSR_IPI_BUF_20..=IOCSR_IPI_BUF_END => {
            if offset + len > IOCSR_IPI_BUF_38 + 8 {
                panic!("ipi writel IOCSR_IPI_BUF, offset = {:#x}, len = {:#x}", offset, len);    
            }
            write_mailbox(pcpu_id, offset, len, val);
        }
        IOCSR_MAIL_SEND => {
            mail_send_iocsr(val);
        }
        IOCSR_ANY_SEND => {
            // any_send(val);
        }
        _ => {
            panic!("Invalid IPI write offset: {:#x}", offset);
        }
    }

    res
}