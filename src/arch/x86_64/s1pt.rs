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
//  Solicey <lzoi_lth@163.com>

use super::paging::{GenericPTE, Level4PageTable, PagingInstr};
use crate::{
    consts::PAGE_SIZE,
    memory::{
        addr::{GuestPhysAddr, HostPhysAddr, PhysAddr},
        MemFlags,
    },
};
use core::fmt;
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    enum MemType {
        Normal = 0,
        Device = 1,
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        0
    }
    fn flags(&self) -> MemFlags {
        MemFlags::READ
    }
    fn is_unused(&self) -> bool {
        false
    }
    fn is_present(&self) -> bool {
        false
    }
    fn set_addr(&mut self, addr: HostPhysAddr) {}
    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) {}
    fn set_table(&mut self, pa: HostPhysAddr) {}
    fn clear(&mut self) {}
    fn is_huge(&self) -> bool {
        false
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct S1PTInstr;

impl PagingInstr for S1PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {}
    fn flush(_vaddr: Option<usize>) {}
}

pub type Stage1PageTable = Level4PageTable<GuestPhysAddr, PageTableEntry, S1PTInstr>;
