// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! Hypercall numbers shared by hvisor and hvisor-tool.

#[repr(u64)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum HyperCallCode {
    HvVirtioInit = 0,
    HvVirtioInjectIrq = 1,
    HvVirtioGetIrq = 86,
    HvZoneStart = 2,
    HvZoneShutdown = 3,
    HvZoneList = 4,
    HvClearInjectIrq = 11,
    HvIvcInfo = 5,
    HvConfigCheck = 6,
    HvVirtioPCI = 7,
    HvZoneSetBootMode = 8,
    HvGetEcamBase = 9,
}

impl TryFrom<u64> for HyperCallCode {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::HvVirtioInit),
            1 => Ok(Self::HvVirtioInjectIrq),
            86 => Ok(Self::HvVirtioGetIrq),
            2 => Ok(Self::HvZoneStart),
            3 => Ok(Self::HvZoneShutdown),
            4 => Ok(Self::HvZoneList),
            11 => Ok(Self::HvClearInjectIrq),
            5 => Ok(Self::HvIvcInfo),
            6 => Ok(Self::HvConfigCheck),
            7 => Ok(Self::HvVirtioPCI),
            8 => Ok(Self::HvZoneSetBootMode),
            9 => Ok(Self::HvGetEcamBase),
            _ => Err(()),
        }
    }
}
