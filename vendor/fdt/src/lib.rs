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

//! # `fdt`
//!
//! A pure-Rust `#![no_std]` crate for parsing Flattened Devicetrees, with the goal of having a
//! very ergonomic and idiomatic API.
//!
//! [![crates.io](https://img.shields.io/crates/v/fdt.svg)](https://crates.io/crates/fdt) [![Documentation](https://docs.rs/fdt/badge.svg)](https://docs.rs/fdt) ![Build](https://github.com/repnop/fdt/actions/workflows/test.yml/badge.svg?branch=master&event=push)
//!
//! ## License
//!
//! This crate is licensed under the Mozilla Public License 2.0 (see the LICENSE file).
//!
//! ## Example
//!
//! ```rust,no_run
//! static MY_FDT: &[u8] = include_bytes!("../dtb/test.dtb");
//!
//! fn main() {
//!     let fdt = fdt::Fdt::new(MY_FDT).unwrap();
//!
//!     println!("This is a devicetree representation of a {}", fdt.root().model());
//!     println!("...which is compatible with at least: {}", fdt.root().compatible().first());
//!     println!("...and has {} CPU(s)", fdt.cpus().count());
//!     println!(
//!         "...and has at least one memory location at: {:#X}\n",
//!         fdt.memory().regions().next().unwrap().starting_address as usize
//!     );
//!
//!     let chosen = fdt.chosen();
//!     if let Some(bootargs) = chosen.bootargs() {
//!         println!("The bootargs are: {:?}", bootargs);
//!     }
//!
//!     if let Some(stdout) = chosen.stdout() {
//!         println!("It would write stdout to: {}", stdout.name);
//!     }
//!
//!     let soc = fdt.find_node("/soc");
//!     println!("Does it have a `/soc` node? {}", if soc.is_some() { "yes" } else { "no" });
//!     if let Some(soc) = soc {
//!         println!("...and it has the following children:");
//!         for child in soc.children() {
//!             println!("    {}", child.name);
//!         }
//!     }
//! }
//! ```

#![no_std]

#[cfg(test)]
mod tests;

pub mod node;
mod parsing;
pub mod standard_nodes;

#[cfg(feature = "pretty-printing")]
mod pretty_print;

use node::MemoryReservation;
use parsing::{BigEndianU32, CStr, FdtData};
use standard_nodes::{Aliases, Chosen, Cpu, Memory, MemoryRegion, Root};

/// Possible errors when attempting to create an `Fdt`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FdtError {
    /// The FDT had an invalid magic value
    BadMagic,
    /// The given pointer was null
    BadPtr,
    /// The slice passed in was too small to fit the given total size of the FDT
    /// structure
    BufferTooSmall,
}

impl core::fmt::Display for FdtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FdtError::BadMagic => write!(f, "bad FDT magic value"),
            FdtError::BadPtr => write!(f, "an invalid pointer was passed"),
            FdtError::BufferTooSmall => {
                write!(f, "the given buffer was too small to contain a FDT header")
            }
        }
    }
}

/// A flattened devicetree located somewhere in memory
///
/// Note on `Debug` impl: by default the `Debug` impl of this struct will not
/// print any useful information, if you would like a best-effort tree print
/// which looks similar to `dtc`'s output, enable the `pretty-printing` feature
#[derive(Clone, Copy)]
pub struct Fdt<'a> {
    data: &'a [u8],
    header: FdtHeader,
}

impl core::fmt::Debug for Fdt<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "pretty-printing")]
        pretty_print::print_node(f, self.root().node, 0)?;

        #[cfg(not(feature = "pretty-printing"))]
        f.debug_struct("Fdt").finish_non_exhaustive()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct FdtHeader {
    /// FDT header magic
    magic: BigEndianU32,
    /// Total size in bytes of the FDT structure
    totalsize: BigEndianU32,
    /// Offset in bytes from the start of the header to the structure block
    off_dt_struct: BigEndianU32,
    /// Offset in bytes from the start of the header to the strings block
    off_dt_strings: BigEndianU32,
    /// Offset in bytes from the start of the header to the memory reservation
    /// block
    off_mem_rsvmap: BigEndianU32,
    /// FDT version
    version: BigEndianU32,
    /// Last compatible FDT version
    last_comp_version: BigEndianU32,
    /// System boot CPU ID
    boot_cpuid_phys: BigEndianU32,
    /// Length in bytes of the strings block
    size_dt_strings: BigEndianU32,
    /// Length in bytes of the struct block
    size_dt_struct: BigEndianU32,
}

impl FdtHeader {
    fn valid_magic(&self) -> bool {
        self.magic.get() == 0xd00dfeed
    }

    fn struct_range(&self) -> core::ops::Range<usize> {
        let start = self.off_dt_struct.get() as usize;
        let end = start + self.size_dt_struct.get() as usize;

        start..end
    }

    fn strings_range(&self) -> core::ops::Range<usize> {
        let start = self.off_dt_strings.get() as usize;
        let end = start + self.size_dt_strings.get() as usize;

        start..end
    }

    fn from_bytes(bytes: &mut FdtData<'_>) -> Option<Self> {
        Some(Self {
            magic: bytes.u32()?,
            totalsize: bytes.u32()?,
            off_dt_struct: bytes.u32()?,
            off_dt_strings: bytes.u32()?,
            off_mem_rsvmap: bytes.u32()?,
            version: bytes.u32()?,
            last_comp_version: bytes.u32()?,
            boot_cpuid_phys: bytes.u32()?,
            size_dt_strings: bytes.u32()?,
            size_dt_struct: bytes.u32()?,
        })
    }
}

impl<'a> Fdt<'a> {
    /// Construct a new `Fdt` from a byte buffer
    ///
    /// Note: this function does ***not*** require that the data be 4-byte
    /// aligned
    pub fn new(data: &'a [u8]) -> Result<Self, FdtError> {
        let mut stream = FdtData::new(data);
        let header = FdtHeader::from_bytes(&mut stream).ok_or(FdtError::BufferTooSmall)?;

        if !header.valid_magic() {
            return Err(FdtError::BadMagic);
        } else if data.len() < header.totalsize.get() as usize {
            return Err(FdtError::BufferTooSmall);
        }

        Ok(Self { data, header })
    }

    /// # Safety
    /// This function performs a read to verify the magic value. If the pointer
    /// is invalid this can result in undefined behavior.
    ///
    /// Note: this function does ***not*** require that the data be 4-byte
    /// aligned
    pub unsafe fn from_ptr(ptr: *const u8) -> Result<Self, FdtError> {
        if ptr.is_null() {
            return Err(FdtError::BadPtr);
        }

        let tmp_header = core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtHeader>());
        let real_size =
            FdtHeader::from_bytes(&mut FdtData::new(tmp_header)).unwrap().totalsize.get() as usize;

        Self::new(core::slice::from_raw_parts(ptr, real_size))
    }

    /// Return the `/aliases` node, if one exists
    pub fn aliases(&self) -> Option<Aliases<'_, 'a>> {
        Some(Aliases {
            node: node::find_node(&mut FdtData::new(self.structs_block()), "/aliases", self, None)?,
            header: self,
        })
    }

    /// Searches for the `/chosen` node, which is always available
    pub fn chosen(&self) -> Chosen<'_, 'a> {
        node::find_node(&mut FdtData::new(self.structs_block()), "/chosen", self, None)
            .map(|node| Chosen { node })
            .expect("/chosen is required")
    }

    /// Return the `/cpus` node, which is always available
    pub fn cpus(&self) -> impl Iterator<Item = Cpu<'_, 'a>> {
        let parent = self.find_node("/cpus").expect("/cpus is a required node");

        parent
            .children()
            .filter(|c| c.name.split('@').next().unwrap() == "cpu")
            .map(move |cpu| Cpu { parent, node: cpu })
    }

    /// Returns the memory node, which is always available
    pub fn memory(&self) -> Memory<'_, 'a> {
        Memory { node: self.find_node("/memory").expect("requires memory node") }
    }

    /// Returns an iterator over the memory reservations
    pub fn memory_reservations(&self) -> impl Iterator<Item = MemoryReservation> + 'a {
        let mut stream = FdtData::new(&self.data[self.header.off_mem_rsvmap.get() as usize..]);
        let mut done = false;

        core::iter::from_fn(move || {
            if stream.is_empty() || done {
                return None;
            }

            let res = MemoryReservation::from_bytes(&mut stream)?;

            if res.address() as usize == 0 && res.size() == 0 {
                done = true;
                return None;
            }

            Some(res)
        })
    }

    /// Return the root (`/`) node, which is always available
    pub fn root(&self) -> Root<'_, 'a> {
        Root { node: self.find_node("/").expect("/ is a required node") }
    }

    /// Returns the first node that matches the node path, if you want all that
    /// match the path, use `find_all_nodes`. This will automatically attempt to
    /// resolve aliases if `path` is not found.
    ///
    /// Node paths must begin with a leading `/` and are ASCII only. Passing in
    /// an invalid node path or non-ASCII node name in the path will return
    /// `None`, as they will not be found within the devicetree structure.
    ///
    /// Note: if the address of a node name is left out, the search will find
    /// the first node that has a matching name, ignoring the address portion if
    /// it exists.
    pub fn find_node(&self, path: &str) -> Option<node::FdtNode<'_, 'a>> {
        let node = node::find_node(&mut FdtData::new(self.structs_block()), path, self, None);
        node.or_else(|| self.aliases()?.resolve_node(path))
    }

    /// Searches for a node which contains a `compatible` property and contains
    /// one of the strings inside of `with`
    pub fn find_compatible(&self, with: &[&str]) -> Option<node::FdtNode<'_, 'a>> {
        self.all_nodes().find(|n| {
            n.compatible().and_then(|compats| compats.all().find(|c| with.contains(c))).is_some()
        })
    }

    /// Searches for the given `phandle`
    pub fn find_phandle(&self, phandle: u32) -> Option<node::FdtNode<'_, 'a>> {
        self.all_nodes().find(|n| {
            n.properties()
                .find(|p| p.name == "phandle")
                .and_then(|p| Some(BigEndianU32::from_bytes(p.value)?.get() == phandle))
                .unwrap_or(false)
        })
    }

    /// Returns an iterator over all of the available nodes with the given path.
    /// This does **not** attempt to find any node with the same name as the
    /// provided path, if you're looking to do that, [`Fdt::all_nodes`] will
    /// allow you to iterate over each node's name and filter for the desired
    /// node(s).
    ///
    /// For example:
    /// ```rust
    /// static MY_FDT: &[u8] = include_bytes!("../dtb/test.dtb");
    ///
    /// let fdt = fdt::Fdt::new(MY_FDT).unwrap();
    ///
    /// for node in fdt.find_all_nodes("/soc/virtio_mmio") {
    ///     println!("{}", node.name);
    /// }
    /// ```
    /// prints:
    /// ```notrust
    /// virtio_mmio@10008000
    /// virtio_mmio@10007000
    /// virtio_mmio@10006000
    /// virtio_mmio@10005000
    /// virtio_mmio@10004000
    /// virtio_mmio@10003000
    /// virtio_mmio@10002000
    /// virtio_mmio@10001000
    /// ```
    pub fn find_all_nodes(&self, path: &'a str) -> impl Iterator<Item = node::FdtNode<'_, 'a>> {
        let mut done = false;
        let only_root = path == "/";
        let valid_path = path.chars().fold(0, |acc, c| acc + if c == '/' { 1 } else { 0 }) >= 1;

        let mut path_split = path.rsplitn(2, '/');
        let child_name = path_split.next().unwrap();
        let parent = match path_split.next() {
            Some("") => Some(self.root().node),
            Some(s) => node::find_node(&mut FdtData::new(self.structs_block()), s, self, None),
            None => None,
        };

        let (parent, bad_parent) = match parent {
            Some(parent) => (parent, false),
            None => (self.find_node("/").unwrap(), true),
        };

        let mut child_iter = parent.children();

        core::iter::from_fn(move || {
            if done || !valid_path || bad_parent {
                return None;
            }

            if only_root {
                done = true;
                return self.find_node("/");
            }

            let mut ret = None;

            #[allow(clippy::while_let_on_iterator)]
            while let Some(child) = child_iter.next() {
                if child.name.split('@').next()? == child_name {
                    ret = Some(child);
                    break;
                }
            }

            ret
        })
    }

    /// Returns an iterator over all of the nodes in the devicetree, depth-first
    pub fn all_nodes(&self) -> impl Iterator<Item = node::FdtNode<'_, 'a>> {
        node::all_nodes(self)
    }

    /// Returns an iterator over all of the strings inside of the strings block
    pub fn strings(&self) -> impl Iterator<Item = &'a str> {
        let mut block = self.strings_block();

        core::iter::from_fn(move || {
            if block.is_empty() {
                return None;
            }

            let cstr = CStr::new(block)?;

            block = &block[cstr.len() + 1..];

            cstr.as_str()
        })
    }

    /// Total size of the devicetree in bytes
    pub fn total_size(&self) -> usize {
        self.header.totalsize.get() as usize
    }

    fn cstr_at_offset(&self, offset: usize) -> CStr<'a> {
        CStr::new(&self.strings_block()[offset..]).expect("no null terminating string on C str?")
    }

    fn str_at_offset(&self, offset: usize) -> &'a str {
        self.cstr_at_offset(offset).as_str().expect("not utf-8 cstr")
    }

    fn strings_block(&self) -> &'a [u8] {
        &self.data[self.header.strings_range()]
    }

    fn structs_block(&self) -> &'a [u8] {
        &self.data[self.header.struct_range()]
    }
}
