// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! SCMI ioctl ABI shared with hvisor-tool.

pub const HVISOR_SCMI_CLOCK_GET_COUNT: u32 = 0x01;
pub const HVISOR_SCMI_CLOCK_GET_ATTRIBUTES: u32 = 0x02;
pub const HVISOR_SCMI_CLOCK_DESCRIBE_RATES: u32 = 0x03;
pub const HVISOR_SCMI_CLOCK_RATE_GET: u32 = 0x04;
pub const HVISOR_SCMI_CLOCK_RATE_SET: u32 = 0x05;
pub const HVISOR_SCMI_CLOCK_CONFIG_GET: u32 = 0x06;
pub const HVISOR_SCMI_CLOCK_CONFIG_SET: u32 = 0x07;
pub const HVISOR_SCMI_CLOCK_NAME_GET: u32 = 0x08;

pub const HVISOR_SCMI_RESET_GET_COUNT: u32 = 0x01;
pub const HVISOR_SCMI_RESET_RESET: u32 = 0x02;

pub const HVISOR_SCMI_POWER_GET_COUNT: u32 = 0x01;
pub const HVISOR_SCMI_POWER_GET_ATTRIBUTES: u32 = 0x02;
pub const HVISOR_SCMI_POWER_STATE_SET: u32 = 0x03;
pub const HVISOR_SCMI_POWER_STATE_GET: u32 = 0x04;

pub const SCMI_POWER_STATE_GENERIC_ON: u32 = 0x00000000;
pub const SCMI_POWER_STATE_GENERIC_OFF: u32 = 0x40000000;

pub const HVISOR_SCMI_CLOCK_IOCTL: u32 = 0xC058_0120;
pub const HVISOR_SCMI_RESET_IOCTL: u32 = 0xC014_0121;
pub const HVISOR_SCMI_POWER_IOCTL: u32 = 0xC050_0122;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockAttr {
    pub clock_id: u32,
    pub enabled: u32,
    pub parent_id: u32,
    pub clock_name: [u8; 64],
    pub is_valid: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockRatesInfo {
    pub clock_id: u32,
    pub rate_index: u32,
    pub num_rates: u32,
    pub remaining: u32,
    pub rates: [u64; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockRateInfo {
    pub clock_id: u32,
    pub rate: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockRateSetInfo {
    pub clock_id: u32,
    pub flags: u32,
    pub rate: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockConfigInfo {
    pub clock_id: u32,
    pub flags: u32,
    pub config: u32,
    pub extended_config_val: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockNameInfo {
    pub clock_id: u32,
    pub name: [u8; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HvScmiClockData {
    pub clock_count: u32,
    pub clock_attr: HvScmiClockAttr,
    pub clock_rates_info: HvScmiClockRatesInfo,
    pub clock_rate_info: HvScmiClockRateInfo,
    pub clock_rate_set_info: HvScmiClockRateSetInfo,
    pub clock_config_info: HvScmiClockConfigInfo,
    pub clock_name_info: HvScmiClockNameInfo,
    pub data: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiClockArgs {
    pub subcmd: u32,
    pub data_len: u32,
    pub u: HvScmiClockData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiResetInfo {
    pub domain_id: u32,
    pub flags: u32,
    pub reset_state: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HvScmiResetData {
    pub reset_count: u32,
    pub reset_info: HvScmiResetInfo,
    pub data: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiResetArgs {
    pub subcmd: u32,
    pub data_len: u32,
    pub u: HvScmiResetData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiPowerAttr {
    pub domain_id: u32,
    pub flags: u32,
    pub name: [u8; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiPowerStateInfo {
    pub domain_id: u32,
    pub power_state: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HvScmiPowerData {
    pub power_count: u32,
    pub power_attr: HvScmiPowerAttr,
    pub power_state_info: HvScmiPowerStateInfo,
    pub data: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HvScmiPowerArgs {
    pub subcmd: u32,
    pub data_len: u32,
    pub u: HvScmiPowerData,
}
