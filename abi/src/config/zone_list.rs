// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! Zone-list hypercall ABI.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    pub zone_id: u32,
    pub cpus: u64,
    pub name: [u8; 32],
    pub is_err: u8,
}
