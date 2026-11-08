// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! Ioctl commands and user/kernel data shared with hvisor-tool.
//!
//! The numeric values match Linux ioctl encoding on the 64-bit targets
//! currently supported by hvisor.

use crate::config::ZoneInfo;

pub const HVISOR_INIT_VIRTIO: u32 = 0x0000_0100;
pub const HVISOR_GET_TASK: u32 = 0x0000_0101;
pub const HVISOR_FINISH_REQ: u32 = 0x0000_0102;
pub const HVISOR_ZONE_START: u32 = 0x4008_0103;
pub const HVISOR_ZONE_SHUTDOWN: u32 = 0x4008_0104;
pub const HVISOR_ZONE_LIST: u32 = 0x8008_0105;
pub const HVISOR_CONFIG_CHECK: u32 = 0x8008_0106;
pub const HVISOR_SET_EVENTFD: u32 = 0x4004_0107;
pub const HVISOR_LOAD_IMAGE: u32 = 0x4018_0108;
pub const HVISOR_SET_BOOT_MODE: u32 = 0x4008_0109;
pub const HVISOR_GET_ECAM_BASE: u32 = 0x8008_010a;
pub const HVISOR_DEASSERT_IRQ: u32 = 0x4008_010b;
pub const HVISOR_IVC_USER_INFO: u32 = 0x8008_4900;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HvZoneListArgs {
    pub cnt: u64,
    pub zones: *mut ZoneInfo,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HvLoadImageArgs {
    pub user_buffer: u64,
    pub size: u64,
    pub load_paddr: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HvIrqLineArgs {
    pub zone_id: u32,
    pub irq_id: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HvIvcUserInfo {
    pub len: i32,
    pub ivc_ids: [i32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HvIvcControlTable {
    pub ivc_id: u32,
    pub max_peers: u32,
    pub rw_sec_size: u32,
    pub out_sec_size: u32,
    pub peer_id: u32,
    pub ipi_invoke: u32,
}
