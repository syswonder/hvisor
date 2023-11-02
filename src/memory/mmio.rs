use crate::{error::HvResult, percpu::this_cell};

use super::GuestPhysAddr;

pub type MMIOHandler = fn(&MMIOAccess) -> HvResult;

pub struct MMIOAccess {
    /** Address to access, depending on the context, an absolute address or
     * relative offset to region start. */
    pub address: GuestPhysAddr,
    /** Size of the access. */
    pub size: u64,
    /** True if write access. */
    pub is_write: bool,
    /** The value to be written or the read value to return. */
    pub value: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct MMIORegion {
    pub start: GuestPhysAddr,
    pub size: u64,
}

pub struct MMIOConfig {
    pub region: MMIORegion,
    pub handler: MMIOHandler,
}

impl MMIORegion {
    pub fn contains_region(&self, addr: GuestPhysAddr, sz: u64) -> bool {
        addr >= self.start && addr + (sz as usize) <= self.start + (self.size as usize)
    }
}

#[allow(unused)]
pub fn mmio_handle_access(mmio: &MMIOAccess) -> HvResult {
    let cell = this_cell();
    let cell_r = cell.read();
    match cell_r.find_mmio_region(mmio.address, mmio.size) {
        Some(x) => {
            println!("x={:x?}", x.0);
        }
        None => todo!(),
    }
    loop {}
}
