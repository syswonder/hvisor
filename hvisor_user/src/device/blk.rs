const BLK_SIZE_MAX: u32 = 524288; // 128 * 4096
const BLK_SEG_MAX: u32 = 512;
// Feature bits
pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1; // Indicates maximum segment size
pub const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;  // Indicates maximum # of segments

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtioBlkConfig {
    capacity: usize,
    size_max: u32,
    seg_max: u32,
    geometry: BlkGeometry,
    blk_size: usize,
    topology: BlkTopology,
    writeback: u8,
    unused0: [u8; 3],
    max_discard_sectors: u32,
    max_discard_seg: u32,
    discard_sector_alignment: u32,
    max_write_zeroes_sectors: u32,
    max_write_zeroes_seg: u32,
    write_zeroes_may_unmap: u8,
    unused1: [u8; 3],
}

impl VirtioBlkConfig {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity,
            size_max: BLK_SIZE_MAX,
            seg_max: BLK_SEG_MAX,
            geometry: BlkGeometry::default(),
            blk_size: 0,
            topology: BlkTopology::default(),
            writeback: 0,
            unused0: [0; 3],
            max_discard_sectors: 0,
            max_discard_seg: 0,
            discard_sector_alignment: 0,
            max_write_zeroes_sectors: 0,
            max_write_zeroes_seg: 0,
            write_zeroes_may_unmap: 0,
            unused1: [0; 3],
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone)]
struct BlkGeometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

impl Default for BlkGeometry {
    fn default() -> Self {
        Self {
            cylinders: 0,
            heads: 0,
            sectors: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct BlkTopology {
    // # of logical blocks per physical block (log2)
    physical_block_exp: u8,
    // offset of first aligned logical block
    alignment_offset: u8,
    // suggested minimum I/O size in blocks
    min_io_size: u16,
    // optimal (suggested maximum) I/O size in blocks
    opt_io_size: u32,
}

impl Default for BlkTopology {
    fn default() -> Self {
        Self {
            physical_block_exp: 0,
            alignment_offset: 0,
            min_io_size: 0,
            opt_io_size: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VirtioBlkReqHead {
    req_type: u32,
    reserved: u32,
    sector: u64,
}