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
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    node::{CellSizes, FdtNode, NodeProperty},
    parsing::{BigEndianU32, BigEndianU64, CStr, FdtData},
    Fdt,
};

/// Represents the `/chosen` node with specific helper methods
#[derive(Debug, Clone, Copy)]
pub struct Chosen<'b, 'a: 'b> {
    pub(crate) node: FdtNode<'b, 'a>,
}

impl<'b, 'a: 'b> Chosen<'b, 'a> {
    /// Contains the bootargs, if they exist
    pub fn bootargs(self) -> Option<&'a str> {
        self.node
            .properties()
            .find(|n| n.name == "bootargs")
            .and_then(|n| core::str::from_utf8(&n.value[..n.value.len() - 1]).ok())
    }

    /// Searches for the node representing `stdout`, if the property exists,
    /// attempting to resolve aliases if the node name doesn't exist as-is
    pub fn stdout(self) -> Option<FdtNode<'b, 'a>> {
        self.node
            .properties()
            .find(|n| n.name == "stdout-path")
            .and_then(|n| core::str::from_utf8(&n.value[..n.value.len() - 1]).ok())
            .and_then(|name| self.node.header.find_node(name))
    }

    /// Searches for the node representing `stdout`, if the property exists,
    /// attempting to resolve aliases if the node name doesn't exist as-is. If
    /// no `stdin` property exists, but `stdout` is present, it will return the
    /// node specified by the `stdout` property.
    pub fn stdin(self) -> Option<FdtNode<'b, 'a>> {
        self.node
            .properties()
            .find(|n| n.name == "stdin-path")
            .and_then(|n| core::str::from_utf8(&n.value[..n.value.len() - 1]).ok())
            .and_then(|name| self.node.header.find_node(name))
            .or_else(|| self.stdout())
    }
}

/// Represents the root (`/`) node with specific helper methods
#[derive(Debug, Clone, Copy)]
pub struct Root<'b, 'a: 'b> {
    pub(crate) node: FdtNode<'b, 'a>,
}

impl<'b, 'a: 'b> Root<'b, 'a> {
    /// Root node cell sizes
    pub fn cell_sizes(self) -> CellSizes {
        self.node.cell_sizes()
    }

    /// `model` property
    pub fn model(self) -> &'a str {
        self.node
            .properties()
            .find(|p| p.name == "model")
            .and_then(|p| core::str::from_utf8(p.value).map(|s| s.trim_end_matches('\0')).ok())
            .unwrap()
    }

    /// `compatible` property
    pub fn compatible(self) -> Compatible<'a> {
        self.node.compatible().unwrap()
    }

    /// Returns an iterator over all of the available properties
    pub fn properties(self) -> impl Iterator<Item = NodeProperty<'a>> + 'b {
        self.node.properties()
    }

    /// Attempts to find the a property by its name
    pub fn property(self, name: &str) -> Option<NodeProperty<'a>> {
        self.node.properties().find(|p| p.name == name)
    }
}

/// Represents the `/aliases` node with specific helper methods
#[derive(Debug, Clone, Copy)]
pub struct Aliases<'b, 'a: 'b> {
    pub(crate) header: &'b Fdt<'a>,
    pub(crate) node: FdtNode<'b, 'a>,
}

impl<'b, 'a: 'b> Aliases<'b, 'a> {
    /// Attempt to resolve an alias to a node name
    pub fn resolve(self, alias: &str) -> Option<&'a str> {
        self.node
            .properties()
            .find(|p| p.name == alias)
            .and_then(|p| core::str::from_utf8(p.value).map(|s| s.trim_end_matches('\0')).ok())
    }

    /// Attempt to find the node specified by the given alias
    pub fn resolve_node(self, alias: &str) -> Option<FdtNode<'b, 'a>> {
        self.resolve(alias).and_then(|name| self.header.find_node(name))
    }

    /// Returns an iterator over all of the available aliases
    pub fn all(self) -> impl Iterator<Item = (&'a str, &'a str)> + 'b {
        self.node.properties().filter_map(|p| {
            Some((p.name, core::str::from_utf8(p.value).map(|s| s.trim_end_matches('\0')).ok()?))
        })
    }
}

/// Represents a `/cpus/cpu*` node with specific helper methods
#[derive(Debug, Clone, Copy)]
pub struct Cpu<'b, 'a: 'b> {
    pub(crate) parent: FdtNode<'b, 'a>,
    pub(crate) node: FdtNode<'b, 'a>,
}

impl<'b, 'a: 'b> Cpu<'b, 'a> {
    /// Return the IDs for the given CPU
    pub fn ids(self) -> CpuIds<'a> {
        let address_cells = self.node.parent_cell_sizes().address_cells;

        CpuIds {
            reg: self
                .node
                .properties()
                .find(|p| p.name == "reg")
                .expect("reg is a required property of cpu nodes"),
            address_cells,
        }
    }

    /// `clock-frequency` property
    pub fn clock_frequency(self) -> usize {
        self.node
            .properties()
            .find(|p| p.name == "clock-frequency")
            .or_else(|| self.parent.property("clock-frequency"))
            .map(|p| match p.value.len() {
                4 => BigEndianU32::from_bytes(p.value).unwrap().get() as usize,
                8 => BigEndianU64::from_bytes(p.value).unwrap().get() as usize,
                _ => unreachable!(),
            })
            .expect("clock-frequency is a required property of cpu nodes")
    }

    /// `timebase-frequency` property
    pub fn timebase_frequency(self) -> usize {
        self.node
            .properties()
            .find(|p| p.name == "timebase-frequency")
            .or_else(|| self.parent.property("timebase-frequency"))
            .map(|p| match p.value.len() {
                4 => BigEndianU32::from_bytes(p.value).unwrap().get() as usize,
                8 => BigEndianU64::from_bytes(p.value).unwrap().get() as usize,
                _ => unreachable!(),
            })
            .expect("timebase-frequency is a required property of cpu nodes")
    }

    /// Returns an iterator over all of the properties for the CPU node
    pub fn properties(self) -> impl Iterator<Item = NodeProperty<'a>> + 'b {
        self.node.properties()
    }

    /// Attempts to find the a property by its name
    pub fn property(self, name: &str) -> Option<NodeProperty<'a>> {
        self.node.properties().find(|p| p.name == name)
    }
}

/// Represents the value of the `reg` property of a `/cpus/cpu*` node which may
/// contain more than one CPU or thread ID
#[derive(Debug, Clone, Copy)]
pub struct CpuIds<'a> {
    pub(crate) reg: NodeProperty<'a>,
    pub(crate) address_cells: usize,
}

impl<'a> CpuIds<'a> {
    /// The first listed CPU ID, which will always exist
    pub fn first(self) -> usize {
        match self.address_cells {
            1 => BigEndianU32::from_bytes(self.reg.value).unwrap().get() as usize,
            2 => BigEndianU64::from_bytes(self.reg.value).unwrap().get() as usize,
            n => panic!("address-cells of size {} is currently not supported", n),
        }
    }

    /// Returns an iterator over all of the listed CPU IDs
    pub fn all(self) -> impl Iterator<Item = usize> + 'a {
        let mut vals = FdtData::new(self.reg.value);
        core::iter::from_fn(move || match vals.remaining() {
            [] => None,
            _ => Some(match self.address_cells {
                1 => vals.u32()?.get() as usize,
                2 => vals.u64()?.get() as usize,
                n => panic!("address-cells of size {} is currently not supported", n),
            }),
        })
    }
}

/// Represents the `compatible` property of a node
#[derive(Clone, Copy)]
pub struct Compatible<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> Compatible<'a> {
    /// First compatible string
    pub fn first(self) -> &'a str {
        CStr::new(self.data).expect("expected C str").as_str().unwrap()
    }

    /// Returns an iterator over all available compatible strings
    pub fn all(self) -> impl Iterator<Item = &'a str> {
        let mut data = self.data;
        core::iter::from_fn(move || {
            if data.is_empty() {
                return None;
            }

            match data.iter().position(|b| *b == b'\0') {
                Some(idx) => {
                    let ret = Some(core::str::from_utf8(&data[..idx]).ok()?);
                    data = &data[idx + 1..];

                    ret
                }
                None => {
                    let ret = Some(core::str::from_utf8(data).ok()?);
                    data = &[];

                    ret
                }
            }
        })
    }
}

/// Represents the `/memory` node with specific helper methods
#[derive(Debug, Clone, Copy)]
pub struct Memory<'b, 'a: 'b> {
    pub(crate) node: FdtNode<'b, 'a>,
}

impl Memory<'_, '_> {
    /// Returns an iterator over all of the available memory regions
    pub fn regions(&self) -> impl Iterator<Item = MemoryRegion> + '_ {
        self.node.reg().unwrap()
    }

    /// Returns the initial mapped area, if it exists
    pub fn initial_mapped_area(&self) -> Option<MappedArea> {
        let mut mapped_area = None;

        if let Some(init_mapped_area) = self.node.property("initial_mapped_area") {
            let mut stream = FdtData::new(init_mapped_area.value);
            let effective_address = stream.u64().expect("effective address");
            let physical_address = stream.u64().expect("physical address");
            let size = stream.u32().expect("size");

            mapped_area = Some(MappedArea {
                effective_address: effective_address.get() as usize,
                physical_address: physical_address.get() as usize,
                size: size.get() as usize,
            });
        }

        mapped_area
    }
}

/// An area described by the `initial-mapped-area` property of the `/memory`
/// node
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct MappedArea {
    /// Effective address of the mapped area
    pub effective_address: usize,
    /// Physical address of the mapped area
    pub physical_address: usize,
    /// Size of the mapped area
    pub size: usize,
}

/// A memory region
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryRegion {
    /// Starting address represented as a pointer
    pub starting_address: *const u8,
    /// Size of the memory region
    pub size: Option<usize>,
}
