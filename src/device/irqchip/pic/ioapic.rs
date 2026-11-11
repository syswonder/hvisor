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
        acpi::try_get_cpu_id,
        cpu::{this_apic_id, this_cpu_id},
        zone::HvArchZoneConfig,
    },
    cpu_data::this_zone,
    device::irqchip::pic::inject_vector,
    device::irqchip::pic::ioapic_state::{Delivery, IoApicState},
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    platform::ROOT_ZONE_IOAPIC_BASE,
    zone::{this_zone_id, Zone},
};
use alloc::vec::Vec;
use bit_field::BitField;
use spin::{Mutex, Once};
use x2apic::ioapic::IoApic;
use x86_64::instructions::port::Port;

pub mod irqs {
    pub const UART_COM1_IRQ: u8 = 0x4;
}

#[allow(non_snake_case)]
pub mod IoApicReg {
    pub const ID: u32 = 0x00;
    pub const VERSION: u32 = 0x01;
    pub const ARBITRATION: u32 = 0x02;
    pub const TABLE_BASE: u32 = 0x10;
}

const IOAPIC_MAX_REDIRECT_ENTRIES: u64 = 0x17;

lazy_static::lazy_static! {
    static ref IO_APIC: Mutex<IoApic> = {
        unsafe { Mutex::new(IoApic::new(ROOT_ZONE_IOAPIC_BASE as _)) }
    };
}

static VIRT_IOAPIC: Once<VirtIoApic> = Once::new();

#[derive(Default)]
struct VirtIoApicUnlocked {
    cur_reg: u32,
    state: IoApicState,
}

pub struct VirtIoApic {
    inner: Vec<Mutex<VirtIoApicUnlocked>>,
}

impl VirtIoApic {
    pub fn new(max_zones: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_zones {
            let v = Mutex::new(VirtIoApicUnlocked::default());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn read(&self, gpa: GuestPhysAddr) -> HvResult<u64> {
        // info!("ioapic read! gpa: {:x}", gpa,);
        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();

        if gpa == 0 {
            return Ok(ioapic.lock().cur_reg as _);
        }
        assert!(gpa == 0x10);

        let inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID => Ok(0),
            IoApicReg::VERSION => Ok(IOAPIC_MAX_REDIRECT_ENTRIES << 16 | 0x11), // max redirect entries: 0x17, version: 0x11
            IoApicReg::ARBITRATION => Ok(0),
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                // info!("ioapic read index: {:x}", index);
                if this_zone_id() != 0 && index == 4 {
                    //FIXME: we do not allow non-root to use uart interrupt
                    return Ok(0xffff_ffff_ffff_ffff);
                }
                if let Some(entry) = inner.state.entry(index) {
                    if reg % 2 == 0 {
                        Ok(entry.get_bits(0..=31))
                    } else {
                        Ok(entry.get_bits(32..=63))
                    }
                } else {
                    Ok(0)
                }
            }
        }
    }

    fn write(&self, gpa: GuestPhysAddr, value: u64, size: usize) -> HvResult {
        /*info!(
            "ioapic write! gpa: {:x}, value: {:x}, size: {:x}",
            gpa, value, size,
        );*/

        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();
        if gpa == 0 {
            ioapic.lock().cur_reg = value as _;
            return Ok(());
        }
        assert!(gpa == 0x10);

        let mut inner = ioapic.lock();
        let mut replay = None;
        match inner.cur_reg {
            IoApicReg::ID | IoApicReg::VERSION | IoApicReg::ARBITRATION => {}
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.state.entry(index) {
                    let value = if reg % 2 == 0 {
                        value as u32
                    } else {
                        let original_dest = value.get_bits(24..=31);
                        let dest = if let Some(dest_cpu) = try_get_cpu_id(original_dest as usize) {
                            if this_zone().read().cpu_set().contains_cpu(dest_cpu) {
                                original_dest
                            } else {
                                this_apic_id() as u64
                            }
                        } else {
                            this_apic_id() as u64
                        };
                        (value as u32 & 0x00ff_ffff) | ((dest as u32) << 24)
                    };
                    replay = inner.state.write_half(index, reg % 2 != 0, value);
                    if zone_id == 0 {
                        let entry = inner.state.entry(index).unwrap_or(entry);
                        unsafe { configure_gsi_from_raw(index as _, entry) };
                    }
                }
            }
        }
        if let Some(delivery) = replay {
            Self::inject_delivery(delivery, false);
        }
        Ok(())
    }

    fn inject_delivery(delivery: Delivery, allow_repeat: bool) {
        let dest =
            try_get_cpu_id(delivery.entry.get_bits(56..=63) as usize).unwrap_or_else(this_cpu_id);
        inject_vector(
            dest,
            delivery.entry as u8,
            None,
            allow_repeat,
            delivery.level,
        );
    }

    fn get_irq_cpu(&self, irq: usize, zone_id: usize) -> Option<usize> {
        let ioapic = self.inner.get(zone_id)?;
        let entry = ioapic.lock().state.entry(irq)?;
        Some(try_get_cpu_id(entry.get_bits(56..=63) as usize).unwrap_or_else(this_cpu_id))
    }

    fn pulse(&self, irq: usize, allow_repeat: bool) {
        let ioapic = self.inner.get(this_zone_id()).unwrap();
        let mut inner = ioapic.lock();
        let delivery = inner.state.pulse(irq);
        if let Some(delivery) = delivery {
            Self::inject_delivery(delivery, allow_repeat);
        }
    }

    fn set_line(&self, zone_id: usize, irq: usize, high: bool) -> bool {
        let Some(ioapic) = self.inner.get(zone_id) else {
            return false;
        };
        let mut inner = ioapic.lock();
        if inner.state.entry(irq).is_none() {
            return false;
        }
        let delivery = inner.state.set_line(irq, high);
        if let Some(delivery) = delivery {
            Self::inject_delivery(delivery, false);
        }
        true
    }

    fn eoi(&self, zone_id: usize, vector: u8) {
        let Some(ioapic) = self.inner.get(zone_id) else {
            return;
        };
        let mut inner = ioapic.lock();
        let deliveries = inner.state.eoi(vector);
        for delivery in deliveries.into_iter().flatten() {
            Self::inject_delivery(delivery, false);
        }
    }

    fn reset(&self, zone_id: usize) {
        if let Some(ioapic) = self.inner.get(zone_id) {
            ioapic.lock().state = IoApicState::default();
        }
    }

    fn begin_shutdown(&self, zone_id: usize) {
        if let Some(ioapic) = self.inner.get(zone_id) {
            ioapic.lock().state.begin_shutdown();
        }
    }
}

impl Zone {
    pub fn ioapic_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        if arch.ioapic_base == 0 || arch.ioapic_size == 0 {
            return;
        }
        self.write().mmio_region_register(
            arch.ioapic_base,
            arch.ioapic_size,
            mmio_ioapic_handler,
            arch.ioapic_base,
        );
    }
}

fn mmio_ioapic_handler(mmio: &mut MMIOAccess, _: usize) -> HvResult {
    if mmio.is_write {
        VIRT_IOAPIC
            .get()
            .unwrap()
            .write(mmio.address, mmio.value as _, mmio.size)
    } else {
        mmio.value = VIRT_IOAPIC.get().unwrap().read(mmio.address).unwrap() as _;
        Ok(())
    }
}

unsafe fn configure_gsi_from_raw(irq: u8, raw: u64) {
    // info!("irq={:x} {:x}", irq, raw);
    let mut io_apic = IO_APIC.lock();
    io_apic.set_table_entry(irq, core::mem::transmute(raw));
}

pub fn init_ioapic() {
    // println!("Initializing I/O APIC...");
    unsafe {
        Port::<u8>::new(0x20).write(0xff);
        Port::<u8>::new(0xa0).write(0xff);
    }
}

pub fn init_virt_ioapic(max_zones: usize) {
    VIRT_IOAPIC.call_once(|| VirtIoApic::new(max_zones));
}

pub fn ioapic_inject_irq(irq: u8, allow_repeat: bool) {
    VIRT_IOAPIC.get().unwrap().pulse(irq as _, allow_repeat);
}

pub fn get_irq_cpu(irq: usize, zone_id: usize) -> usize {
    VIRT_IOAPIC
        .get()
        .unwrap()
        .get_irq_cpu(irq, zone_id)
        .unwrap()
}

pub fn ioapic_set_line(zone_id: usize, irq: usize, high: bool) -> bool {
    VIRT_IOAPIC.get().unwrap().set_line(zone_id, irq, high)
}

pub fn ioapic_eoi(zone_id: usize, vector: u8) {
    VIRT_IOAPIC.get().unwrap().eoi(zone_id, vector);
}

pub fn ioapic_reset(zone_id: usize) {
    VIRT_IOAPIC.get().unwrap().reset(zone_id);
}

pub fn ioapic_begin_shutdown(zone_id: usize) {
    VIRT_IOAPIC.get().unwrap().begin_shutdown(zone_id);
}
