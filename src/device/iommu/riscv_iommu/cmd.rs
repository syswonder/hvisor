// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Jingyu Liu <liujingyu24s@ict.ac.cn>

#![allow(unused)]

pub(super) const IOTINVAL_OPCODE: u8 = 1;
pub(super) const IOFENCE_OPCODE: u8 = 2;
pub(super) const IODIR_OPCODE: u8 = 3;

// Dword0 and dword1 are the two 64-bit words that make up the command.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub(super) struct RiscvIommuCommand {
    pub dword0: u64,
    pub dword1: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommandBuildError {
    FieldOutOfRange(&'static str, u64),
    MisalignedField(&'static str, u64, u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IotInvalFunc {
    Vma,
    Gvma,
}

impl IotInvalFunc {
    pub(super) fn raw(self) -> u8 {
        match self {
            Self::Vma => 0,
            Self::Gvma => 1,
        }
    }
}

/// RISC-V IOMMU Spec v1.0 Chapter 4.1.1 IOMMU Page-Table cache invalidation commands
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IotInvalCommand {
    pub func: IotInvalFunc, // [9:7]
    pub av: bool, // [10], The AV operand indicates if the address (ADDR) operand is valid.
    pub pscid: u32, // [31:12]
    pub pscv: bool, // [32], The PSCV operand indicates if the Process Soft-Context ID (PSCID) operand is valid.
    pub gv: bool, // [33], The GV operand indicates if the Guest-Soft-Context ID (GSCID) operand is valid.
    pub nl: bool, // [34], The definition of the NL bit is provided by the non-leaf PTE invalidation extension
    pub gscid: u16, // [59:44]
    pub s: bool, // [73], The definition of the S bit is provided by the address range invalidation extension
    pub addr: u64, // [125:74]
}

impl IotInvalCommand {
    /// Encode the command into two 64-bit words.
    pub(super) fn encode(self) -> Result<RiscvIommuCommand, CommandBuildError> {
        // Check if the pscid is valid.
        if self.pscid >= (1 << 20) {
            return Err(CommandBuildError::FieldOutOfRange(
                "iotinval.pscid",
                self.pscid as u64,
            ));
        }
        let mut dword0 = u64::from(IOTINVAL_OPCODE) | (u64::from(self.func.raw()) << 7);
        if self.av {
            dword0 |= 1u64 << 10;
        }
        dword0 |= (u64::from(self.pscid) & 0xFFFFF) << 12;
        if self.pscv {
            dword0 |= 1u64 << 32;
        }
        if self.gv {
            dword0 |= 1u64 << 33;
        }
        if self.nl {
            dword0 |= 1u64 << 34;
        }
        dword0 |= u64::from(self.gscid) << 44;
        // Command[125:74] <- ADDR[63:12], i.e. dword1[61:10] <- addr[63:12].
        let mut dword1 = ((self.addr >> 12) & ((1u64 << 52) - 1)) << 10;
        if self.s {
            dword1 |= 1u64 << 9;
        }
        Ok(RiscvIommuCommand { dword0, dword1 })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IoFenceFunc {
    C,
}

impl IoFenceFunc {
    pub(super) fn raw(self) -> u8 {
        match self {
            Self::C => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IoFenceCommand {
    pub func: IoFenceFunc, // [9:7]
    pub av: bool,          // [10]
    pub wsi: bool,         // [11]
    pub pr: bool,          // [12]
    pub pw: bool,          // [13]
    pub data: u32,         // [63:32]
    pub addr: u64,         // [125:64]
}

impl IoFenceCommand {
    pub(super) fn encode(self) -> Result<RiscvIommuCommand, CommandBuildError> {
        let mut dword0 = u64::from(IOFENCE_OPCODE) | (u64::from(self.func.raw()) << 7);
        if self.av {
            dword0 |= 1u64 << 10;
        }
        if self.wsi {
            dword0 |= 1u64 << 11;
        }
        if self.pr {
            dword0 |= 1u64 << 12;
        }
        if self.pw {
            dword0 |= 1u64 << 13;
        }
        dword0 |= u64::from(self.data) << 32;
        // Command[125:64] <- ADDR[63:2], i.e. dword1[61:0] <- addr[63:2].
        let dword1 = self.addr >> 2;
        Ok(RiscvIommuCommand { dword0, dword1 })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IoDirFunc {
    InvalDdt,
    InvalPdt,
}

impl IoDirFunc {
    pub(super) fn raw(self) -> u8 {
        match self {
            Self::InvalDdt => 0,
            Self::InvalPdt => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IoDirCommand {
    pub func: IoDirFunc, // [9:7]
    pub pid: u32,        // [31:12]
    pub dv: bool,        // [33]
    pub did: u32,        // [63:40]
}

impl IoDirCommand {
    pub(super) fn encode(self) -> Result<RiscvIommuCommand, CommandBuildError> {
        if self.pid >= (1 << 20) {
            return Err(CommandBuildError::FieldOutOfRange(
                "iodir.pid",
                self.pid as u64,
            ));
        }
        if self.did >= (1 << 24) {
            return Err(CommandBuildError::FieldOutOfRange(
                "iodir.did",
                self.did as u64,
            ));
        }
        let mut dword0 = u64::from(IODIR_OPCODE) | (u64::from(self.func.raw()) << 7);
        dword0 |= u64::from(self.pid & 0xFFFFF) << 12;
        if self.dv {
            dword0 |= 1u64 << 33;
        }
        dword0 |= u64::from(self.did & 0xFFFFFF) << 40;
        Ok(RiscvIommuCommand { dword0, dword1: 0 })
    }
}
