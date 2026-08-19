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
use crate::consts::{IPI_EVENT_CLEAR_INJECT_IRQ, IPI_EVENT_SEND_IPI};
use core::arch::asm;
use core::ptr::write_volatile;
use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::*;

pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    debug!(
        "loongarch64: arch_send_event: sending event to cpu: {}, sgi_num: {}",
        cpu_id, sgi_num
    );
    // Route hvisor event doorbells through the IOCSR sender, whose payload
    // carries the target CPU ID instead of indexing the fixed legacy table.
    ipi_write_action_percore(cpu_id as usize, sgi_num as usize);
}

pub fn arch_notify_event(cpu_id: u64, sgi_num: u64, event_id: usize, queue_was_empty: bool) {
    debug!(
        "loongarch64: notify event: cpu={}, ipi={}, event={}, queue_was_empty={}",
        cpu_id, sgi_num, event_id, queue_was_empty
    );
    if queue_was_empty {
        arch_send_event(cpu_id, sgi_num);
    }
}

const MMIO_BASE: usize = 0x8000_0000_1fe0_0000;
const IOCSR_IPI_STATUS: usize = 0x1000;
const IOCSR_IPI_ENABLE: usize = 0x1004;
const IOCSR_IPI_CLEAR: usize = 0x100c;

#[inline]
fn iocsr_read32(reg: usize) -> u32 {
    let value: usize;
    unsafe {
        asm!("iocsrrd.w {}, {}", out(reg) value, in(reg) reg);
    }
    value as u32
}

#[inline]
fn iocsr_write32(value: u32, reg: usize) {
    unsafe {
        asm!("iocsrwr.w {}, {}", in(reg) value as usize, in(reg) reg);
    }
}

// ipi actions
pub const SMP_BOOT_CPU: usize = 0x1;
pub const SMP_RESCHEDULE: usize = 0x2;
pub const SMP_CALL_FUNCTION: usize = 0x4;
// customized actions :), since there is no docs on this yet
/// Dedicated physical IPI bit used only as the hvisor event-queue doorbell.
/// Linux SMP actions use bits 0..=2, so sharing those bits can drop guest IPI work.
pub const HVISOR_EVENT_DOORBELL: usize = 0x8;

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

pub fn enable_ipi() {
    iocsr_write32(u32::MAX, IOCSR_IPI_ENABLE);
    debug!("enable_ipi: IPI enabled for cpu {}", this_cpu_id());
}

pub fn clear_all_ipi() {
    iocsr_write32(u32::MAX, IOCSR_IPI_CLEAR);
    debug!(
        "clear_all_ipi: IPI status for cpu {}: {:#x}",
        this_cpu_id(),
        iocsr_read32(IOCSR_IPI_STATUS)
    );
}

pub fn clear_ipi_bits(mask: u32) {
    iocsr_write32(mask, IOCSR_IPI_CLEAR);
}

pub fn reset_ipi() {
    // clear all IPIs and enable all IPIs
    clear_all_ipi();
    enable_ipi();
}

pub fn get_ipi_status() -> u32 {
    iocsr_read32(IOCSR_IPI_STATUS)
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

pub fn arch_check_events(event: Option<usize>) {
    match event {
        Some(IPI_EVENT_CLEAR_INJECT_IRQ) => {
            warn!("legacy CLEAR_INJECT_IRQ event ignored; use the per-IRQ line API");
        }
        Some(IPI_EVENT_SEND_IPI) => {
            crate::arch::zone::sync_virtual_ipi_line();
        }
        _ => {
            panic!("arch_check_events: unhandled event: {:?}", event);
        }
    }
}
