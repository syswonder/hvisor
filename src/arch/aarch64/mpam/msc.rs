#![allow(unused)]

#[repr(C, packed)]
#[derive(Debug)]
pub struct MscNode {
    length: u16,
    interface_type: u8, // 0x00 = MMIO, 0x0A = PCC
    reserved: u8,
    identifier: u32,
    base_address: u64,
    mmio_size: u32,
    overflow_interrupt: u32,
    overflow_interrupt_flags: u32,
    reserved1: u32,
    overflow_interrupt_affinity: u32,
    error_interrupt: u32,
    error_interrupt_flags: u32,
    reserved2: u32,
    error_interrupt_affinity: u32,
    maxn_rdy_usec: u32,
    hardware_id_of_linked_device: u64,
    instance_id_of_linked_device: u32,
    number_of_resource_nodes: u32,
    // followed by resource nodes
}

impl MscNode {
    pub fn length(&self) -> u16 {
        self.length
    }
    pub fn identifier(&self) -> u32 {
        self.identifier
    }
    pub fn interface_type(&self) -> u8 {
        self.interface_type
    }
    #[inline(always)]
    pub fn base_address(&self) -> u64 {
        self.base_address
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct ResourceNode {
    identifier: u32,
    ris_index: u8,
    reserved1: u16,
    locator_type: u8,
    locator: [u8; 12],
    number_of_functional_dependencies: u32,
    // followed by functional dependency descriptors
}

impl ResourceNode {
    #[inline(always)]
    pub fn identifier(&self) -> u32 {
        self.identifier
    }
    #[inline(always)]
    pub fn ris_index(&self) -> u8 {
        self.ris_index
    }
    #[inline(always)]
    pub fn locator_type(&self) -> u8 {
        self.locator_type
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct FunctionalDependencyDescriptor {
    pub producer: u32,
    pub reserved: u32,
}

pub fn mpam_get_msc_node_base_address_iter() -> impl Iterator<Item = u64> {
    MPAM_NODES.iter().map(|(msc_node, _)| msc_node.base_address)
}

pub static MPAM_NODES: &[(MscNode, &[ResourceNode])] = &[
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 0,
            base_address: 0xB01A000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 1,
        },
        &[ResourceNode {
            identifier: 0,
            ris_index: 0,
            reserved1: 0,
            locator_type: 0,
            locator: [
                0x03, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            number_of_functional_dependencies: 0,
        }],
    ),
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 1,
            base_address: 0xB016000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 1,
        },
        &[ResourceNode {
            identifier: 1,
            ris_index: 0,
            reserved1: 0,
            locator_type: 0,
            locator: [
                0x02, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            number_of_functional_dependencies: 0,
        }],
    ),
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 2,
            base_address: 0xB012000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 1,
        },
        &[ResourceNode {
            identifier: 2,
            ris_index: 0,
            reserved1: 0,
            locator_type: 0,
            locator: [
                0x01, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            number_of_functional_dependencies: 0,
        }],
    ),
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 3,
            base_address: 0xB00E000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 1,
        },
        &[ResourceNode {
            identifier: 3,
            ris_index: 0,
            reserved1: 0,
            locator_type: 0,
            locator: [
                0x00, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            number_of_functional_dependencies: 0,
        }],
    ),
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 4,
            base_address: 0xB00A000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 1,
        },
        &[ResourceNode {
            identifier: 4,
            ris_index: 0,
            reserved1: 0,
            locator_type: 0,
            locator: [
                0x00, 0x00, 0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            number_of_functional_dependencies: 0,
        }],
    ),
    (
        MscNode {
            length: 0,
            interface_type: 0,
            reserved: 0,
            identifier: 5,
            base_address: 0xB006000,
            mmio_size: 16384,
            overflow_interrupt: 44,
            overflow_interrupt_flags: 1,
            reserved1: 0,
            overflow_interrupt_affinity: 0,
            error_interrupt: 45,
            error_interrupt_flags: 1,
            reserved2: 0,
            error_interrupt_affinity: 0,
            maxn_rdy_usec: 100,
            hardware_id_of_linked_device: 0,
            instance_id_of_linked_device: 0,
            number_of_resource_nodes: 4,
        },
        &[
            ResourceNode {
                identifier: 5,
                ris_index: 0,
                reserved1: 0,
                locator_type: 1,
                locator: [0x00; 12],
                number_of_functional_dependencies: 0,
            },
            ResourceNode {
                identifier: 5,
                ris_index: 1,
                reserved1: 0,
                locator_type: 1,
                locator: [
                    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                number_of_functional_dependencies: 0,
            },
            ResourceNode {
                identifier: 5,
                ris_index: 2,
                reserved1: 0,
                locator_type: 1,
                locator: [
                    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                number_of_functional_dependencies: 0,
            },
            ResourceNode {
                identifier: 5,
                ris_index: 3,
                reserved1: 0,
                locator_type: 1,
                locator: [
                    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                number_of_functional_dependencies: 0,
            },
        ],
    ),
];
