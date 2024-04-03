use crate::{
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
};

pub fn mmio_pci_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    mmio_perform_access(base, mmio);
    Ok(())
}
