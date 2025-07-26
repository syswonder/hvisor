use crate::{
    arch::{cpu::this_cpu_id, idt, ipi, mmio::MMIoDevice, zone::HvArchZoneConfig},
    device::irqchip::pic::inject_vector,
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    platform::ROOT_ZONE_IOAPIC_BASE,
    zone::{this_zone_id, Zone},
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
        assert!(gpa == 0x10);

        let inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID => Ok(0),
            IoApicReg::VERSION => Ok(IOAPIC_MAX_REDIRECT_ENTRIES << 16 | 0x11), // max redirect entries: 0x17, version: 0x11
            IoApicReg::ARBITRATION => Ok(0),
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.rte.get(index) {
                    if reg % 2 == 0 {
                        let mut lower = (*entry).get_bits(0..=31);
                        if let Some(gv) =
                            idt::get_guest_vector(lower.get_bits(0..=7) as u8, zone_id)
                        {
                            lower.set_bits(0..=7, gv as _);
                        }
                        Ok(lower.get_bits(0..=31))
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
            ioapic.lock().cur_reg = value as _;
            return Ok(());
        }
        assert!(gpa == 0x10);

        let mut inner = ioapic.lock();
        match inner.cur_reg {
            IoApicReg::ID | IoApicReg::VERSION | IoApicReg::ARBITRATION => {}
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.rte.get_mut(index) {
                    if reg % 2 == 0 {
                        entry.set_bits(0..=31, value.get_bits(0..=31));
                        // use host vector instead of guest vector
                        let gv = entry.get_bits(0..=7) as u32;
                        if let Some(hv) = idt::get_host_vector(gv, zone_id) {
                            entry.set_bits(0..=7, hv as _);
                        }
                    } else {
                        entry.set_bits(32..=63, value.get_bits(0..=31));

                        /*if zone_id == 0 {
                            // info!("1 write {:x} entry: {:x?}", index, *entry);
                            // only root zone modify the real I/O APIC
                            // unsafe { configure_gsi_from_raw(index as _, *entry) };
                        }*/
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
        let ioapic = self.inner.get(zone_id).unwrap();
        if let Some(entry) = ioapic.lock().rte.get(irq) {
            let dest = entry.get_bits(56..=63) as usize;
            return Some(dest);
        }
        None
    }

    fn trigger(&self, irq: usize, allow_repeat: bool) -> HvResult {
        let zone_id = this_zone_id();
        let ioapic = self.inner.get(zone_id).unwrap();
        if let Some(entry) = ioapic.lock().rte.get(irq) {
            // TODO: physical & logical mode
            let dest = entry.get_bits(56..=63) as usize;
            let masked = entry.get_bit(16);
            let vector = entry.get_bits(0..=7) as u8;
            // info!("trigger hv: {:x} zone: {:x}", vector, zone_id);
            if let Some(gv) = idt::get_guest_vector(vector as _, zone_id) {
                if !masked {
                    inject_vector(dest, gv as _, None, allow_repeat);
                }
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
        self.mmio_region_register(
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
    println!("Initializing I/O APIC...");
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
        .unwrap()
}
