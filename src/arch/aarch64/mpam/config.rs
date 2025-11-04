#[derive(Debug)]
pub struct MpamPartConfig {
    pub partid: u32,
    // Cache partition: specified as a percentage (0–100)
    pub cache_percentage: Option<u32>,
    // Memory bandwidth partition: specified in MB/s
    pub mem_max_bw: Option<u32>,
    pub mem_min_bw: Option<u32>,
}

pub static TOTAL_MEM_BW: usize = 9600; 

pub static MPAM_SYSTEM_CONFIG: &[MpamPartConfig] = &[
    MpamPartConfig {
        partid: 0,
        cache_percentage: Some(50),
        mem_max_bw: Some(4800),
        mem_min_bw: Some(0),
    },
    MpamPartConfig {
        partid: 1,
        cache_percentage: Some(10),
        mem_max_bw: Some(1200),
        mem_min_bw: Some(1000),
    },
    MpamPartConfig {
        partid: 2,
        cache_percentage: Some(40),
        mem_max_bw: Some(600),
        mem_min_bw: Some(600),
    },
];
