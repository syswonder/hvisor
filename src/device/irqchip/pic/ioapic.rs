use crate::{
    arch::{
        mmio::{mmio_handler, MMIoDevice},
        zone::HvArchZoneConfig,
    },
    device::irqchip::pic::inject_vector,
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    platform::ROOT_ZONE_IOAPIC_BASE,
    zone::Zone,
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{ops::Range, u32};
use spin::Mutex;
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

lazy_static::lazy_static! {
    static ref VIRT_IOAPIC: (Arc<dyn MMIoDevice>,) = (Arc::new(VirtIoApic::new(ROOT_ZONE_IOAPIC_BASE)),);
}

#[derive(Default)]
struct VirtIoApicUnlocked {
    cur_reg: u32,
    rte: [u64; (IOAPIC_MAX_REDIRECT_ENTRIES + 1) as usize],
}

pub struct VirtIoApic {
    base_gpa: usize,
    gpa_range: Vec<Range<usize>>,
    inner: Mutex<VirtIoApicUnlocked>,
}

impl VirtIoApic {
    pub fn new(base_gpa: GuestPhysAddr) -> Self {
        Self {
            base_gpa,
            gpa_range: vec![base_gpa..base_gpa + 0x1000],
            inner: Mutex::new(VirtIoApicUnlocked::default()),
        }
    }
}

impl MMIoDevice for VirtIoApic {
    fn gpa_range(&self) -> &Vec<Range<usize>> {
        &self.gpa_range
    }

    fn read(&self, gpa: GuestPhysAddr) -> HvResult<u64> {
        // info!("ioapic read! gpa: {:x}", gpa,);

        if gpa == self.base_gpa {
            return Ok(self.inner.lock().cur_reg as _);
        }
        assert!(gpa - self.base_gpa == 0x10);

        let inner = self.inner.lock();
        match inner.cur_reg {
            IoApicReg::ID => Ok(0),
            IoApicReg::VERSION => Ok(IOAPIC_MAX_REDIRECT_ENTRIES << 16 | 0x11), // max redirect entries: 0x17, version: 0x11
            IoApicReg::ARBITRATION => Ok(0),
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
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

    fn write(&self, gpa: GuestPhysAddr, value: u64, size: usize) -> HvResult {
        /*info!(
            "ioapic write! gpa: {:x}, value: {:x}, size: {:x}",
            gpa, value, size,
        );*/
        assert!(size == 4);

        if gpa == self.base_gpa {
            self.inner.lock().cur_reg = value as _;
            return Ok(());
        }
        assert!(gpa - self.base_gpa == 0x10);

        let mut inner = self.inner.lock();
        match inner.cur_reg {
            IoApicReg::ID | IoApicReg::VERSION | IoApicReg::ARBITRATION => {}
            mut reg => {
                reg -= IoApicReg::TABLE_BASE;
                let index = (reg >> 1) as usize;
                if let Some(entry) = inner.rte.get_mut(index) {
                    if reg % 2 == 0 {
                        entry.set_bits(0..=31, value.get_bits(0..=31));
                    } else {
                        entry.set_bits(32..=63, value.get_bits(0..=31));

                        // use host vector instead of guest vector
                        /* entry.set_bits(
                            0..=7,
                            get_host_vector(entry.get_bits(0..=7) as u32).unwrap() as _,
                        ); */
                        unsafe {
                            configure_gsi_from_raw(index as _, *entry);
                        };
                    }
                }
            }
        }
        Ok(())
    }

    fn trigger(&self, signal: usize) -> HvResult {
        if let Some(entry) = self.inner.lock().rte.get(signal) {
            // TODO: physical & logical mode
            let dest = entry.get_bits(56..=63) as usize;
            let masked = entry.get_bit(16);
            let vector = entry.get_bits(0..=7) as u8;
            if !masked {
                inject_vector(dest, vector, None, true);
            }
        }
        Ok(())
    }
}

impl Zone {
    pub fn ioapic_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        self.mmio_region_register(arch.ioapic_base, arch.ioapic_size, ioapic_mmio_handler, 0);
    }
}

fn ioapic_mmio_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    mmio_handler(mmio, &VIRT_IOAPIC.0)
}

unsafe fn configure_gsi_from_raw(irq: u8, raw: u64) {
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

pub fn ioapic_inject_irq(irq: u8) {
    VIRT_IOAPIC.0.trigger(irq as _);
}
