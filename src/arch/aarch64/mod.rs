pub mod entry;
pub mod exception;
pub mod sysreg;
pub mod s1pt;
pub mod s2pt;
//mod vcpu;

pub use s1pt::Stage1PageTable;
pub use s2pt::Stage2PageTable;