// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    parsing::{BigEndianU32, BigEndianU64, CStr, FdtData},
    standard_nodes::{Compatible, MemoryRegion},
    Fdt,
};

const FDT_BEGIN_NODE: u32 = 1;
const FDT_END_NODE: u32 = 2;
const FDT_PROP: u32 = 3;
pub(crate) const FDT_NOP: u32 = 4;
const FDT_END: u32 = 5;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct FdtProperty {
    len: BigEndianU32,
    name_offset: BigEndianU32,
}

impl FdtProperty {
    fn from_bytes(bytes: &mut FdtData<'_>) -> Option<Self> {
        let len = bytes.u32()?;
        let name_offset = bytes.u32()?;

        Some(Self { len, name_offset })
    }
}

/// A devicetree node
#[derive(Debug, Clone, Copy)]
pub struct FdtNode<'b, 'a: 'b> {
    pub name: &'a str,
    pub(crate) header: &'b Fdt<'a>,
    props: &'a [u8],
    parent_props: Option<&'a [u8]>,
}

impl<'b, 'a: 'b> FdtNode<'b, 'a> {
    fn new(
        name: &'a str,
        header: &'b Fdt<'a>,
        props: &'a [u8],
        parent_props: Option<&'a [u8]>,
    ) -> Self {
        Self { name, header, props, parent_props }
    }

    /// Returns an iterator over the available properties of the node
    pub fn properties(self) -> impl Iterator<Item = NodeProperty<'a>> + 'b {
        let mut stream = FdtData::new(self.props);
        let mut done = false;

        core::iter::from_fn(move || {
            if stream.is_empty() || done {
                return None;
            }

            while stream.peek_u32()?.get() == FDT_NOP {
                stream.skip(4);
            }

            if stream.peek_u32().unwrap().get() == FDT_PROP {
                Some(NodeProperty::parse(&mut stream, self.header))
            } else {
                done = true;
                None
            }
        })
    }

    /// Attempts to find the a property by its name
    pub fn property(self, name: &str) -> Option<NodeProperty<'a>> {
        self.properties().find(|p| p.name == name)
    }

    /// Returns an iterator over the children of the current node
    pub fn children(self) -> impl Iterator<Item = FdtNode<'b, 'a>> {
        let mut stream = FdtData::new(self.props);

        while stream.peek_u32().unwrap().get() == FDT_NOP {
            stream.skip(4);
        }

        while stream.peek_u32().unwrap().get() == FDT_PROP {
            NodeProperty::parse(&mut stream, self.header);
        }

        let mut done = false;

        core::iter::from_fn(move || {
            if stream.is_empty() || done {
                return None;
            }

            while stream.peek_u32()?.get() == FDT_NOP {
                stream.skip(4);
            }

            if stream.peek_u32()?.get() == FDT_BEGIN_NODE {
                let origin = stream.remaining();
                let ret = {
                    stream.skip(4);
                    let unit_name = CStr::new(stream.remaining()).expect("unit name").as_str()?;
                    let full_name_len = unit_name.len() + 1;
                    stream.skip(full_name_len);

                    if full_name_len % 4 != 0 {
                        stream.skip(4 - (full_name_len % 4));
                    }

                    Some(Self::new(unit_name, self.header, stream.remaining(), Some(self.props)))
                };

                stream = FdtData::new(origin);

                skip_current_node(&mut stream, self.header);

                ret
            } else {
                done = true;
                None
            }
        })
    }

    /// `reg` property
    ///
    /// Important: this method assumes that the value(s) inside the `reg`
    /// property represent CPU-addressable addresses that are able to fit within
    /// the platform's pointer size (e.g. `#address-cells` and `#size-cells` are
    /// less than or equal to 2 for a 64-bit platform). If this is not the case
    /// or you're unsure of whether this applies to the node, it is recommended
    /// to use the [`FdtNode::property`] method to extract the raw value slice
    /// or use the provided [`FdtNode::raw_reg`] helper method to give you an
    /// iterator over the address and size slices. One example of where this
    /// would return `None` for a node is a `pci` child node which contains the
    /// PCI address information in the `reg` property, of which the address has
    /// an `#address-cells` value of 3.
    pub fn reg(self) -> Option<impl Iterator<Item = crate::MemoryRegion> + 'a> {
        let sizes = self.parent_cell_sizes();
        if sizes.address_cells > 2 || sizes.size_cells > 2 {
            return None;
        }

        let mut reg = None;
        for prop in self.properties() {
            if prop.name == "reg" {
                let mut stream = FdtData::new(prop.value);
                reg = Some(core::iter::from_fn(move || {
                    let starting_address = match sizes.address_cells {
                        1 => stream.u32()?.get() as usize,
                        2 => stream.u64()?.get() as usize,
                        _ => return None,
                    } as *const u8;

                    let size = match sizes.size_cells {
                        0 => None,
                        1 => Some(stream.u32()?.get() as usize),
                        2 => Some(stream.u64()?.get() as usize),
                        _ => return None,
                    };

                    Some(MemoryRegion { starting_address, size })
                }));
                break;
            }
        }

        reg
    }

    pub fn ranges(self) -> Option<impl Iterator<Item = crate::MemoryRegion> + 'a> {
        let sizes = self.cell_sizes();
        if sizes.address_cells != 3 || sizes.size_cells != 2 {
            return None;
        }
    
        let mut ranges = None;
        for prop in self.properties() {
            if prop.name == "ranges" {
                let mut stream = FdtData::new(prop.value);
                ranges = Some(core::iter::from_fn(move || {
                    stream.skip(12);
                    let starting_address = stream.u64()?.get() as usize as *const u8;
                    let size = Some(stream.u64()?.get() as usize);
                    Some(MemoryRegion { starting_address, size })
                }));
                break;
            }
        }
    
        ranges
    }


    /// Convenience method that provides an iterator over the raw bytes for the
    /// address and size values inside of the `reg` property
    pub fn raw_reg(self) -> Option<impl Iterator<Item = RawReg<'a>> + 'a> {
        let sizes = self.parent_cell_sizes();

        if let Some(prop) = self.property("reg") {
            let mut stream = FdtData::new(prop.value);
            return Some(core::iter::from_fn(move || {
                Some(RawReg {
                    address: stream.take(sizes.address_cells * 4)?,
                    size: stream.take(sizes.size_cells * 4)?,
                })
            }));
        }

        None
    }

    /// `compatible` property
    pub fn compatible(self) -> Option<Compatible<'a>> {
        let mut s = None;
        for prop in self.properties() {
            if prop.name == "compatible" {
                s = Some(Compatible { data: prop.value });
            }
        }

        s
    }

    /// Cell sizes for child nodes
    pub fn cell_sizes(self) -> CellSizes {
        let mut cell_sizes = CellSizes::default();

        for property in self.properties() {
            match property.name {
                "#address-cells" => {
                    cell_sizes.address_cells = BigEndianU32::from_bytes(property.value)
                        .expect("not enough bytes for #address-cells value")
                        .get() as usize;
                }
                "#size-cells" => {
                    cell_sizes.size_cells = BigEndianU32::from_bytes(property.value)
                        .expect("not enough bytes for #size-cells value")
                        .get() as usize;
                }
                _ => {}
            }
        }

        cell_sizes
    }

    /// Searches for the interrupt parent, if the node contains one
    pub fn interrupt_parent(self) -> Option<FdtNode<'b, 'a>> {
        self.properties()
            .find(|p| p.name == "interrupt-parent")
            .and_then(|p| self.header.find_phandle(BigEndianU32::from_bytes(p.value)?.get()))
    }

    /// `#interrupt-cells` property
    pub fn interrupt_cells(self) -> Option<usize> {
        let mut interrupt_cells = None;

        if let Some(prop) = self.property("#interrupt-cells") {
            interrupt_cells = BigEndianU32::from_bytes(prop.value).map(|n| n.get() as usize)
        }

        interrupt_cells
    }

    /// `interrupts` property
    pub fn interrupts(self) -> Option<impl Iterator<Item = usize> + 'a> {
        let sizes = self.parent_interrupt_cells()?;

        let mut interrupt = None;
        for prop in self.properties() {
            if prop.name == "interrupts" {
                let mut stream = FdtData::new(prop.value);
                interrupt = Some(core::iter::from_fn(move || {
                    let interrupt = match sizes {
                        1 => stream.u32()?.get() as usize,
                        2 => stream.u64()?.get() as usize,
                        3 => {
                            let ret = stream.u64()?.get() as usize;
                            stream.skip(4);
                            ret
                        }
                        _ => return None,
                    };

                    Some(interrupt)
                }));
                break;
            }
        }
        interrupt
    }

    // pcie node property:interrupt-map
    pub fn interrupt_map(self) -> Option<impl Iterator<Item = usize> + 'a> {
        let mut interrupt = None;
        for prop in self.properties() {
            if prop.name == "interrupt-map" {
                let mut stream = FdtData::new(prop.value);
                interrupt = Some(core::iter::from_fn(move || {
                    stream.skip(4*8);

                    let interrupt = stream.u32()?.get() as usize;

                    stream.skip(4);

                    Some(interrupt)
                }));
                break;
            }
        }
        interrupt
    }

    pub(crate) fn parent_cell_sizes(self) -> CellSizes {
        let mut cell_sizes = CellSizes::default();

        if let Some(parent) = self.parent_props {
            let parent =
                FdtNode { name: "", props: parent, header: self.header, parent_props: None };
            cell_sizes = parent.cell_sizes();
        }

        cell_sizes
    }

    pub(crate) fn parent_interrupt_cells(self) -> Option<usize> {
        let mut interrupt_cells = None;
        let parent = self
            .property("interrupt-parent")
            .and_then(|p| self.header.find_phandle(BigEndianU32::from_bytes(p.value)?.get()))
            .or_else(|| {
                Some(FdtNode {
                    name: "",
                    props: self.parent_props?,
                    header: self.header,
                    parent_props: None,
                })
            });

        if let Some(size) = parent.and_then(|parent| parent.interrupt_cells()) {
            interrupt_cells = Some(size);
        }

        interrupt_cells
    }
}

/// The number of cells (big endian u32s) that addresses and sizes take
#[derive(Debug, Clone, Copy)]
pub struct CellSizes {
    /// Size of values representing an address
    pub address_cells: usize,
    /// Size of values representing a size
    pub size_cells: usize,
}

impl Default for CellSizes {
    fn default() -> Self {
        CellSizes { address_cells: 2, size_cells: 1 }
    }
}

/// A raw `reg` property value set
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawReg<'a> {
    /// Big-endian encoded bytes making up the address portion of the property.
    /// Length will always be a multiple of 4 bytes.
    pub address: &'a [u8],
    /// Big-endian encoded bytes making up the size portion of the property.
    /// Length will always be a multiple of 4 bytes.
    pub size: &'a [u8],
}

pub(crate) fn find_node<'b, 'a: 'b>(
    stream: &mut FdtData<'a>,
    name: &str,
    header: &'b Fdt<'a>,
    parent_props: Option<&'a [u8]>,
) -> Option<FdtNode<'b, 'a>> {
    let mut parts = name.splitn(2, '/');
    let looking_for = parts.next()?;

    stream.skip_nops();

    let curr_data = stream.remaining();

    match stream.u32()?.get() {
        FDT_BEGIN_NODE => {}
        _ => return None,
    }

    let unit_name = CStr::new(stream.remaining()).expect("unit name C str").as_str()?;

    let full_name_len = unit_name.len() + 1;
    skip_4_aligned(stream, full_name_len);

    let looking_contains_addr = looking_for.contains('@');
    let addr_name_same = unit_name == looking_for;
    let base_name_same = unit_name.split('@').next()? == looking_for;

    if (looking_contains_addr && !addr_name_same) || (!looking_contains_addr && !base_name_same) {
        *stream = FdtData::new(curr_data);
        skip_current_node(stream, header);

        return None;
    }

    let next_part = match parts.next() {
        None | Some("") => {
            return Some(FdtNode::new(unit_name, header, stream.remaining(), parent_props))
        }
        Some(part) => part,
    };

    stream.skip_nops();

    let parent_props = Some(stream.remaining());

    while stream.peek_u32()?.get() == FDT_PROP {
        let _ = NodeProperty::parse(stream, header);
    }

    while stream.peek_u32()?.get() == FDT_BEGIN_NODE {
        if let Some(p) = find_node(stream, next_part, header, parent_props) {
            return Some(p);
        }
    }

    stream.skip_nops();

    if stream.u32()?.get() != FDT_END_NODE {
        return None;
    }

    None
}

// FIXME: this probably needs refactored
pub(crate) fn all_nodes<'b, 'a: 'b>(header: &'b Fdt<'a>) -> impl Iterator<Item = FdtNode<'b, 'a>> {
    let mut stream = FdtData::new(header.structs_block());
    let mut done = false;
    let mut parents: [&[u8]; 64] = [&[]; 64];
    let mut parent_index = 0;

    core::iter::from_fn(move || {
        if stream.is_empty() || done {
            return None;
        }

        while stream.peek_u32()?.get() == FDT_END_NODE {
            parent_index -= 1;
            stream.skip(4);
        }

        if stream.peek_u32()?.get() == FDT_END {
            done = true;
            return None;
        }

        while stream.peek_u32()?.get() == FDT_NOP {
            stream.skip(4);
        }

        match stream.u32()?.get() {
            FDT_BEGIN_NODE => {}
            _ => return None,
        }

        let unit_name = CStr::new(stream.remaining()).expect("unit name C str").as_str().unwrap();
        let full_name_len = unit_name.len() + 1;
        skip_4_aligned(&mut stream, full_name_len);

        let curr_node = stream.remaining();

        parent_index += 1;
        parents[parent_index] = curr_node;

        while stream.peek_u32()?.get() == FDT_NOP {
            stream.skip(4);
        }

        while stream.peek_u32()?.get() == FDT_PROP {
            NodeProperty::parse(&mut stream, header);
        }

        Some(FdtNode {
            name: if unit_name.is_empty() { "/" } else { unit_name },
            header,
            parent_props: match parent_index {
                1 => None,
                _ => Some(parents[parent_index - 1]),
            },
            props: curr_node,
        })
    })
}

pub(crate) fn skip_current_node<'a>(stream: &mut FdtData<'a>, header: &Fdt<'a>) {
    assert_eq!(stream.u32().unwrap().get(), FDT_BEGIN_NODE, "bad node");

    let unit_name = CStr::new(stream.remaining()).expect("unit_name C str").as_str().unwrap();
    let full_name_len = unit_name.len() + 1;
    skip_4_aligned(stream, full_name_len);

    while stream.peek_u32().unwrap().get() == FDT_PROP {
        NodeProperty::parse(stream, header);
    }

    while stream.peek_u32().unwrap().get() == FDT_BEGIN_NODE {
        skip_current_node(stream, header);
    }

    stream.skip_nops();

    assert_eq!(stream.u32().unwrap().get(), FDT_END_NODE, "bad node");
}

/// A node property
#[derive(Debug, Clone, Copy)]
pub struct NodeProperty<'a> {
    /// Property name
    pub name: &'a str,
    /// Property value
    pub value: &'a [u8],
}

impl<'a> NodeProperty<'a> {
    /// Attempt to parse the property value as a `usize`
    pub fn as_usize(self) -> Option<usize> {
        match self.value.len() {
            4 => BigEndianU32::from_bytes(self.value).map(|i| i.get() as usize),
            8 => BigEndianU64::from_bytes(self.value).map(|i| i.get() as usize),
            _ => None,
        }
    }

    /// Attempt to parse the property value as a `&str`
    pub fn as_str(self) -> Option<&'a str> {
        core::str::from_utf8(self.value).map(|s| s.trim_end_matches('\0')).ok()
    }

    fn parse(stream: &mut FdtData<'a>, header: &Fdt<'a>) -> Self {
        match stream.u32().unwrap().get() {
            FDT_PROP => {}
            other => panic!("bad prop, tag: {}", other),
        }

        let prop = FdtProperty::from_bytes(stream).expect("FDT property");
        let data_len = prop.len.get() as usize;

        let data = &stream.remaining()[..data_len];

        skip_4_aligned(stream, data_len);

        NodeProperty { name: header.str_at_offset(prop.name_offset.get() as usize), value: data }
    }
}

/// A memory reservation
#[derive(Debug)]
#[repr(C)]
pub struct MemoryReservation {
    pub(crate) address: BigEndianU64,
    pub(crate) size: BigEndianU64,
}

impl MemoryReservation {
    /// Pointer representing the memory reservation address
    pub fn address(&self) -> *const u8 {
        self.address.get() as usize as *const u8
    }

    /// Size of the memory reservation
    pub fn size(&self) -> usize {
        self.size.get() as usize
    }

    pub(crate) fn from_bytes(bytes: &mut FdtData<'_>) -> Option<Self> {
        let address = bytes.u64()?;
        let size = bytes.u64()?;

        Some(Self { address, size })
    }
}

fn skip_4_aligned(stream: &mut FdtData<'_>, len: usize) {
    stream.skip((len + 3) & !0x3);
}
