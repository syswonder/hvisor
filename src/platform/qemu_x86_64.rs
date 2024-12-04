use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x00000000;
pub const ROOT_ZONE_CPUS: u64 = 0;

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 0] = [];

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {};
