use spin::Once;

pub mod pci;
pub mod pcibar;
pub mod endpoint;
pub mod bridge;

pub const NUM_BAR_REGS_TYPE0: usize = 6;
pub const NUM_BAR_REGS_TYPE1: usize = 2;
pub const PHANTOM_DEV_HEADER: u32 = 0x77777777u32;

pub static ECAM_BASE: Once<usize> = Once::new();

pub fn init_ecam_base(ecam_base: usize){
    ECAM_BASE.call_once(|| ecam_base);
}

pub fn get_ecam_base() -> usize{
    *ECAM_BASE.get().unwrap() as _
}

pub fn cfg_base(bdf: usize) -> usize{
    get_ecam_base() + (bdf << 12)
}
