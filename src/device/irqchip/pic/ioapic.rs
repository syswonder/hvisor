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
        acpi::{get_apic_id, try_get_cpu_id},
        cpu::this_cpu_id,
        idt, ipi,
        mmio::MMIoDevice,
        zone::HvArchZoneConfig,
    },
    cpu_data::{this_zone, CpuSet},
    device::irqchip::pic::inject_vector,
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    platform::ROOT_ZONE_IOAPIC_BASE,
    zone::{find_zone, this_zone_id, Zone},
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{ops::Range, u32};
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
        // Only IOREGSEL (offset 0) and IOWIN (offset 0x10) exist; any other
        // window offset a guest pokes reads back as zero rather than panicking.
        if gpa != 0x10 {
            return Ok(0);
        }

        let inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID => Ok(0),
            IoApicReg::VERSION => Ok(IOAPIC_MAX_REDIRECT_ENTRIES << 16 | 0x11), // max redirect entries: 0x17, version: 0x11
            IoApicReg::ARBITRATION => Ok(0),
            reg if reg < IoApicReg::TABLE_BASE => Ok(0), // reserved register window
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if zone_id != 0 && index == irqs::UART_COM1_IRQ as usize {
                    // The legacy COM1 UART is owned by the root zone. Present a
                    // well-formed *masked* redirection entry to other zones (mask
                    // bit 16 set, vector/destination zero) rather than an
                    // all-ones value, which would otherwise read back as an
                    // entry with a bogus vector 0xff and delivery mode 7.
                    return Ok(if reg % 2 == 0 { 1u64 << 16 } else { 0 });
                }
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

    fn write(&self, gpa: GuestPhysAddr, value: u64, size: usize) -> HvResult {
        /*info!(
            "ioapic write! gpa: {:x}, value: {:x}, size: {:x}",
            gpa, value, size,
        );*/

        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();
        if gpa == 0 {
            // IOREGSEL is an 8-bit register; keep only the low byte so a read
            // returns the architectural value.
            ioapic.lock().cur_reg = (value as u32) & 0xff;
            return Ok(());
        }
        // Only IOWIN (offset 0x10) is writable here; ignore stray window offsets
        // instead of panicking on guest-controlled MMIO.
        if gpa != 0x10 {
            return Ok(());
        }

        let mut inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID | IoApicReg::VERSION | IoApicReg::ARBITRATION => {}
            reg if reg < IoApicReg::TABLE_BASE => {} // reserved register window
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                // The legacy COM1 UART redirection entry is owned by the root
                // zone. Drop a non-root zone's writes to it so it cannot reclaim
                // or reprogram COM1 delivery (reads already return a masked
                // entry for this index).
                if zone_id != 0 && index == irqs::UART_COM1_IRQ as usize {
                    return Ok(());
                }
                if let Some(entry) = inner.rte.get_mut(index) {
                    // Store the guest's write verbatim so it reads back what it
                    // wrote. Destination containment is enforced at injection
                    // time (see `contain_dest_cpu`), which is the single choke
                    // point for *every* RTE shape — including a guest that writes
                    // only the low dword to unmask a vector while leaving a
                    // default/out-of-zone destination, or that toggles logical
                    // destination mode after a high-dword write.
                    if reg % 2 == 0 {
                        entry.set_bits(0..=31, value.get_bits(0..=31));
                    } else {
                        entry.set_bits(32..=63, value.get_bits(0..=31));
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
        // The legacy COM1 UART is root-owned (its non-root RTE always reads back
        // masked); a non-root zone has no target for it, so report no CPU rather
        // than handing back the zone's first CPU.
        if zone_id != 0 && irq == irqs::UART_COM1_IRQ as usize {
            return None;
        }
        let cpu_set = find_zone(zone_id)?.cpu_set();
        let ioapic = self.inner.get(zone_id)?;
        let entry = *ioapic.lock().rte.get(irq)?;
        // Mirror `trigger`'s injection gate: a masked entry (mask bit 16) or one
        // whose vector (bits 0..=7) is below 0x20 never injects, so it has no
        // destination CPU. This keeps an all-zero or masked RTE from resolving to
        // the zone's first CPU, which would be inconsistent with delivery.
        if entry.get_bit(16) || (entry.get_bits(0..=7) as u8) < 0x20 {
            return None;
        }
        contain_dest_cpu(zone_id, &cpu_set, entry)
    }

    fn trigger(&self, irq: usize, allow_repeat: bool) -> HvResult {
        let zone_id = this_zone_id();
        let cpu_set = this_zone().cpu_set();
        let ioapic = self.inner.get(zone_id).unwrap();
        let entry = ioapic.lock().rte.get(irq).copied();
        if let Some(entry) = entry {
            let masked = entry.get_bit(16);
            let vector = entry.get_bits(0..=7) as u8;
            if !masked && vector >= 0x20 {
                // Containment choke point: resolve the destination to an in-zone
                // CPU with a fallible lookup, never trusting the guest-written
                // destination byte/mode, so a non-root zone cannot deliver an
                // interrupt to a CPU outside itself and a bogus APIC id cannot
                // panic the hypervisor.
                if let Some(dest) = contain_dest_cpu(zone_id, &cpu_set, entry) {
                    inject_vector(dest, vector, None, allow_repeat);
                }
            }
        }
        Ok(())
    }
}

/// Resolve the CPU an IOAPIC redirection entry should deliver to, keeping a
/// non-root zone's interrupts contained.
///
/// The root zone (`zone_id == 0`) programs real hardware APIC ids and is
/// trusted. For a non-root zone only a *physical-mode* entry whose destination
/// APIC id resolves to one of the zone's own CPUs is honored; a logical-mode
/// entry (RTE bit 11), an unknown APIC id, or an out-of-zone CPU is redirected
/// to one of the zone's CPUs. The APIC-id lookup is fallible, so a garbage
/// guest destination can never panic the hypervisor. Returns `None` only for a
/// zone with no usable CPU.
fn contain_dest_cpu(zone_id: usize, cpu_set: &CpuSet, entry: u64) -> Option<usize> {
    if zone_id == 0 {
        return try_get_cpu_id(entry.get_bits(56..=63) as usize);
    }
    if !entry.get_bit(11) {
        // The destination byte (RTE bits 56..=63) is read as the full 8-bit
        // field. In legacy xAPIC physical mode only bits 56..=59 are the APIC id
        // and bits 60..=63 are reserved, while logical mode uses the full byte;
        // reading the full byte is containment-safe here because an out-of-zone
        // or unknown id is clamped to an in-zone CPU below, and it matches the
        // small x2APIC ids this platform actually uses.
        if let Some(cpu) = try_get_cpu_id(entry.get_bits(56..=63) as usize) {
            if cpu_set.contains_cpu(cpu) {
                return Some(cpu);
            }
        }
    }
    cpu_set.first_cpu()
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
    VIRT_IOAPIC.get().unwrap().trigger(irq as _, allow_repeat);
}

pub fn get_irq_cpu(irq: usize, zone_id: usize) -> usize {
    VIRT_IOAPIC
        .get()
        .unwrap()
        .get_irq_cpu(irq, zone_id)
        .or_else(|| find_zone(zone_id).and_then(|z| z.cpu_set().first_cpu()))
        .unwrap_or(0)
}
