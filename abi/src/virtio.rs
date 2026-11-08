// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! Virtio shared-memory ABI.
//!
//! These are the data structures shared with hvisor-tool. Controller state and
//! volatile/MMIO accessors stay in hvisor.

pub const MAX_REQ: u32 = 32;
pub const MAX_DEVS: usize = 8;
pub const MAX_CPUS: usize = 32;
pub const MAX_VQ: usize = 16;
pub const VIRTIO_PCI_HYPERCALL_VERSION: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtqueueAreaInfo {
    pub desc_area: u64,
    pub avail_area: u64,
    pub used_area: u64,
    pub queue_size: u64,
}

impl VirtqueueAreaInfo {
    pub fn new(desc: u64, avail: u64, used: u64, queue_size: u64) -> Self {
        Self {
            desc_area: desc,
            avail_area: avail,
            used_area: used,
            queue_size,
        }
    }

    pub fn dummy() -> Self {
        Self {
            desc_area: 0,
            avail_area: 0,
            used_area: 0,
            queue_size: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigInfo {
    pub features: u64,
    pub dev_id: u16,
    pub num_of_queues: u16,
    pub dtype: u16,
    pub padding: u16,
    pub vqs: [VirtqueueAreaInfo; MAX_VQ],
}

impl VirtioPCIConfigInfo {
    pub fn dummy() -> Self {
        Self {
            features: 0,
            dev_id: 0,
            num_of_queues: 0,
            dtype: 0,
            padding: 0,
            vqs: [VirtqueueAreaInfo::dummy(); MAX_VQ],
        }
    }

    pub fn set_features(&mut self, val: u64) {
        self.features = val;
    }

    pub fn set_num_of_queues(&mut self, val: u16) {
        self.num_of_queues = val;
    }

    pub fn set_dtype(&mut self, val: u16) {
        self.dtype = val;
    }

    pub fn set_vqs(&mut self, idx: usize, val: VirtqueueAreaInfo) {
        if idx < MAX_VQ {
            self.vqs[idx] = val;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigReq {
    pub request_id: u32,
    pub padding: u32,
    pub info: VirtioPCIConfigInfo,
}

impl VirtioPCIConfigReq {
    pub fn new(request_id: u32, info: VirtioPCIConfigInfo) -> Self {
        Self {
            request_id,
            padding: 0,
            info,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigRes {
    pub request_id: u32,
    pub status: u32,
    pub dev_id: u16,
    pub padding: u16,
}

impl VirtioPCIConfigRes {
    pub fn success(request_id: u32, dev_id: u16) -> Self {
        Self {
            request_id,
            status: 0,
            dev_id,
            padding: 0,
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtioPCIHypercallOp {
    None = 0,
    ConfigReqComplete = 1,
    DataReqComplete = 2,
    Reset = 3,
}

impl TryFrom<u16> for VirtioPCIHypercallOp {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ConfigReqComplete),
            2 => Ok(Self::DataReqComplete),
            3 => Ok(Self::Reset),
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIHypercallInfo {
    pub version: u16,
    pub op: u16,
    pub target_cpu: u32,
    pub request_id: u32,
    pub status: u32,
}

impl VirtioPCIHypercallInfo {
    pub const fn new() -> Self {
        Self {
            version: VIRTIO_PCI_HYPERCALL_VERSION,
            op: VirtioPCIHypercallOp::None as u16,
            target_cpu: 0,
            request_id: 0,
            status: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VirtioPCIDataInfo {
    pub dev_id: u16,
    pub queue_id: u16,
    pub cpu_id: u16,
    pub msix_vector_idx: u16,
}

impl VirtioPCIDataInfo {
    pub fn new(dev_id: u16, queue_id: u16, msix_vector_idx: u16, cpu_id: u16) -> Self {
        Self {
            dev_id,
            queue_id,
            cpu_id,
            msix_vector_idx,
        }
    }

    pub fn get_identifier(&self) -> u64 {
        (self.dev_id as u64) | ((self.queue_id as u64) << 16) | ((self.cpu_id as u64) << 32)
    }

    pub fn get_msix_vector_idx(&self) -> u16 {
        self.msix_vector_idx
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VirtioPCIDataReq {
    pub request_id: u32,
    pub padding: u32,
    pub info: VirtioPCIDataInfo,
}

impl VirtioPCIDataReq {
    pub fn new(request_id: u32, info: VirtioPCIDataInfo) -> Self {
        Self {
            request_id,
            padding: 0,
            info,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HvisorDeviceReq {
    pub src_cpu: u64,
    pub address: u64,
    pub size: u64,
    pub value: u64,
    pub src_zone: u32,
    pub is_write: u8,
    pub need_interrupt: u8,
    pub _padding: u16,
}

impl HvisorDeviceReq {
    pub fn new(
        src_cpu: u64,
        address: u64,
        size: u64,
        value: u64,
        src_zone: u32,
        is_write: bool,
        need_interrupt: u8,
    ) -> Self {
        Self {
            src_cpu,
            address,
            size,
            value,
            src_zone,
            is_write: if is_write { 1 } else { 0 },
            need_interrupt,
            _padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HvisorDeviceRes {
    pub target_zone: u32,
    pub irq_id: u32,
}

pub const MAX_PCI_CONFIG_REQ: usize = 4;
pub const MAX_PCI_CONFIG_RES: usize = 4;
pub const MAX_PCI_DATA_REQ: usize = 32;

#[repr(C)]
#[derive(Debug)]
pub struct HvVirtioPCIBridgeRegion {
    pub hypercall_info: VirtioPCIHypercallInfo,
    pub config_req_front: u32,
    pub config_req_rear: u32,
    pub config_res_front: u32,
    pub config_res_rear: u32,
    pub data_req_front: u32,
    pub data_req_rear: u32,
    pub config_req_list: [VirtioPCIConfigReq; MAX_PCI_CONFIG_REQ],
    pub config_res_list: [VirtioPCIConfigRes; MAX_PCI_CONFIG_RES],
    pub data_req_list: [VirtioPCIDataReq; MAX_PCI_DATA_REQ],
}

#[repr(C)]
#[derive(Debug)]
pub struct HvVirtioBridge {
    pub req_front: u32,
    pub req_rear: u32,
    pub res_front: u32,
    pub res_rear: u32,
    pub req_list: [HvisorDeviceReq; MAX_REQ as usize],
    pub res_list: [HvisorDeviceRes; MAX_REQ as usize],
    pub cfg_flags: [u64; MAX_CPUS],
    pub cfg_values: [u64; MAX_CPUS],
    pub mmio_addrs: [u64; MAX_DEVS],
    pub mmio_avail: u8,
    pub need_wakeup: u8,
}
