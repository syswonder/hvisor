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
        idt::IdtVector,
        zone::HvArchZoneConfig,
    },
    cpu_data::this_zone,
    device::irqchip::pic::inject_vector,
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    platform::ROOT_ZONE_IOAPIC_BASE,
    zone::{find_zone, this_zone_id, Zone},
};
use alloc::vec::Vec;
use bit_field::BitField;
use core::u32;
use spin::{Mutex, Once};
use x2apic::ioapic::IoApic;
use x86_64::instructions::port::Port;

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
    rte: [u64; (IOAPIC_MAX_REDIRECT_ENTRIES + 1) as usize],
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
        // Other offsets within the registered MMIO page are not modelled; read
        // them as zero rather than faulting the hypervisor.
        if gpa != 0x10 {
            return Ok(0);
        }

        let inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID => Ok(0),
            IoApicReg::VERSION => Ok(IOAPIC_MAX_REDIRECT_ENTRIES << 16 | 0x11), // max redirect entries: 0x17, version: 0x11
            IoApicReg::ARBITRATION => Ok(0),
            reg if reg < IoApicReg::TABLE_BASE => Ok(0),
            reg => {
                let reg = reg - IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.rte.get(index) {
                    if reg % 2 == 0 {
                        Ok((*entry).get_bits(0..=31))
                    } else {
                        Ok((*entry).get_bits(32..=63))
                    }
                } else {
                    Ok(0)
                }
            }
        }
    }

    fn write(&self, gpa: GuestPhysAddr, value: u64, _size: usize) -> HvResult {
        /*info!(
            "ioapic write! gpa: {:x}, value: {:x}, size: {:x}",
            gpa, value, _size,
        );*/

        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();
        if gpa == 0 {
            ioapic.lock().cur_reg = value as _;
            return Ok(());
        }
        // Other offsets within the registered MMIO page are not modelled; ignore.
        if gpa != 0x10 {
            return Ok(());
        }

        let mut inner = ioapic.lock();
        match inner.cur_reg {
            // ID / VERSION / ARBITRATION and any other non-table register: ignore.
            reg if reg < IoApicReg::TABLE_BASE => {}
            reg => {
                let reg = reg - IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.rte.get_mut(index) {
                    if reg % 2 == 0 {
                        entry.set_bits(0..=31, value.get_bits(0..=31));
                    } else {
                        entry.set_bits(32..=63, value.get_bits(0..=31));
                    }

                    // hvisor does not model logical delivery, and the root zone
                    // mirrors this entry into the real I/O APIC, so a logical or
                    // out-of-zone destination must not survive. Re-evaluate after
                    // every dword write (mode and destination may be written in
                    // either order) and force the entry into physical mode with a
                    // destination CPU owned by this zone unless it already is one;
                    // neither software injection nor real hardware can then target
                    // a CPU outside the zone.
                    let requested = entry.get_bits(56..=63) as usize;
                    let in_zone_physical = !entry.get_bit(11)
                        && try_get_cpu_id(requested)
                            .map_or(false, |cpu| this_zone().cpu_set().contains_cpu(cpu));
                    if !in_zone_physical {
                        entry.set_bit(11, false); // physical destination mode
                        entry.set_bits(56..=63, this_apic_id() as u64);
                    }

                    // The root zone mirrors this entry into the real I/O APIC, so the
                    // delivery semantics must be sanitised too, not just the
                    // destination: force Fixed delivery (bits 8..=10) so a guest
                    // cannot raise NMI/INIT/ExtINT on a physical CPU, clear the
                    // reserved bits, and mask any entry whose vector is below the
                    // first valid IRQ vector (0x20) so it cannot collide with a CPU
                    // exception vector. The same sanitised entry is what software
                    // injection reads back, keeping both delivery paths in agreement.
                    entry.set_bits(8..=10, 0);
                    entry.set_bits(17..=55, 0);
                    // Mask the entry if its vector is invalid (below the first IRQ
                    // vector) or collides with a vector hvisor reserves for its own
                    // use on the physical CPUs: a guest device firing on the virtual
                    // IPI / APIC error / spurious / timer vector would otherwise be
                    // taken by the host IDT and drive hypervisor interrupt handling.
                    let vector = entry.get_bits(0..=7) as u8;
                    let reserved = matches!(
                        vector,
                        IdtVector::VIRT_IPI_VECTOR
                            | IdtVector::APIC_ERROR_VECTOR
                            | IdtVector::APIC_SPURIOUS_VECTOR
                            | IdtVector::APIC_TIMER_VECTOR
                    );
                    if vector < 0x20 || reserved {
                        entry.set_bit(16, true);
                    }

                    if zone_id == 0 {
                        // only root zone modify the real I/O APIC
                        unsafe { configure_gsi_from_raw(index as _, *entry) };
                    }
                }
            }
        }
        Ok(())
    }

    fn get_irq_cpu(&self, irq: usize, zone_id: usize) -> Option<usize> {
        // A written entry is clamped to an in-zone destination, but an entry that
        // was never programmed defaults to APIC id 0, so confirm the resolved CPU
        // actually belongs to the target zone before trusting it.
        let entry = *self.inner.get(zone_id).unwrap().lock().rte.get(irq)?;
        let cpu = try_get_cpu_id(entry.get_bits(56..=63) as usize)?;
        find_zone(zone_id)?
            .cpu_set()
            .contains_cpu(cpu)
            .then_some(cpu)
    }

    fn trigger(&self, irq: usize, allow_repeat: bool) -> HvResult {
        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();
        if let Some(entry) = ioapic.lock().rte.get(irq) {
            let masked = entry.get_bit(16);
            let vector = entry.get_bits(0..=7) as u8;
            if !masked && vector >= 0x20 {
                // Deliver only to a CPU this zone owns: a written entry is clamped
                // in-zone, but an unprogrammed entry resolves to APIC 0, so filter
                // by zone membership and otherwise use the current (in-zone) CPU.
                let dest = try_get_cpu_id(entry.get_bits(56..=63) as usize)
                    .filter(|&cpu| this_zone().cpu_set().contains_cpu(cpu))
                    .unwrap_or_else(this_cpu_id);
                inject_vector(dest, vector, None, allow_repeat);
            }
        }
        Ok(())
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
        mmio.value = VIRT_IOAPIC.get().unwrap().read(mmio.address)? as _;
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
    let _ = VIRT_IOAPIC.get().unwrap().trigger(irq as _, allow_repeat);
}

pub fn get_irq_cpu(irq: usize, zone_id: usize) -> usize {
    // If the redirection entry has no resolvable in-zone destination (e.g. a
    // logical-mode mask), fall back to the target zone's first CPU so routing
    // stays inside the zone and never panics on a guest-controlled value.
    VIRT_IOAPIC
        .get()
        .unwrap()
        .get_irq_cpu(irq, zone_id)
        .or_else(|| find_zone(zone_id).and_then(|z| z.cpu_set().first_cpu()))
        .unwrap_or(0)
}
