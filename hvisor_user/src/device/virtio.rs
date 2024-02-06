use std::{fs::File, os::fd::FromRawFd, num::NonZeroUsize};

use log::info;
use nix::{fcntl::{open, OFlag}, sys::{stat::Mode, mman::{mmap, ProtFlags, MapFlags}}};

use super::{blk::VirtioBlkConfig, virtqueue::VirtQueue, mmio::VirtMmioRegs};


/// Maximum requests num for req ring.
pub const MAX_REQ: u32 = 4;
const MEM_PHYS_ADDR: usize = 0x70000000;
const MEM_SIZE: usize = 0x8000000;
static mut MEM_VIRT_ADDR: Option<usize> = None;

// pub static mut HVISOR_REGION: &'static HvisorDeviceRegion;
pub static HVISOR_DEVICE_REGION: Option<&'static HvisorDeviceRegion> = None;

/// El0 and EL2 shared region for virtio requests and results.
#[repr(C)]
pub struct HvisorDeviceRegion {
    idx: u32,
    /// Only el0 updates last_req_idx
    pub last_req_idx: u32,
    /// req ring for el0 and el2 communication
    pub req_list: [HvisorDeviceReq; MAX_REQ as usize],
}

/// Hvisor device requests
#[repr(C)]
pub struct HvisorDeviceReq {
    src_cpu: u64,
    address: u64,
    size: u64,
    value: u64,
    src_cell: u32,
    is_wirte: u8,
    is_cfg: u8,
}



pub enum VirtioDeviceType {
    VirtioTNone = 0,
    VirtioTNet = 1,
    VirtioTBlock = 2,
}

enum VirtioConfig {
    BlkConfig(VirtioBlkConfig),
}

pub struct VirtDev {
    features: u64,
    dev_type: VirtioDeviceType,
    config: VirtioConfig,
    activated: bool,
}

impl VirtDev {
    fn new(dev_type: VirtioDeviceType) -> Self {
        match dev_type {
            VirtioDeviceType::VirtioTBlock => {
                Self {
                    features: 
                }
            }
        }
    }
}
pub struct VirtIODevice {
    id: usize,
    cell_id: usize,
    base_addr: usize,
    irq_id: u32,
    regs: VirtMmioRegs,
    dev: VirtDev,
    vqs: Vec<VirtQueue>,
}

impl VirtIODevice {
    fn new(dev_type: VirtioDeviceType, base_addr: usize, irq_id: u32) -> Self {

    }
}
fn init_virtio_devices() {
    let img_fd = open("virtio_ext4.img", OFlag::O_RDWR, Mode::empty()).unwrap();
    let mem_fd = open("/dev/mem", OFlag::O_RDWR | OFlag::O_SYNC, Mode::empty()).unwrap();
    unsafe {
        let res = mmap(
            None,
            NonZeroUsize::new(MEM_SIZE).unwrap(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            Some(File::from_raw_fd(mem_fd)),
            0,
        ).unwrap();
        info!("mem virt addr is {:#x}", res as usize);
        MEM_VIRT_ADDR = Some(res as usize);
    }

}