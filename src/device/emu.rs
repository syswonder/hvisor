use spin::Mutex;

use super::{
    gicv3::{inject_irq, IRQHVI},
    virtio::{VirtioReq, VIRTIO_REQ_LIST},
};
const MAX_REQ: usize = 4;
pub static HVISOR_DEVICE: Mutex<HvisorDevice> = Mutex::new(HvisorDevice::default());
pub struct HvisorDevice {
    base_address: usize, // el1 and el2 shared region addr, el2 virtual address
    is_enable: bool,
}

impl HvisorDevice {
    fn region(&self) -> &mut HvisorDeviceRegion {
        if !self.is_enable {
            panic!("hvisor device region is not enabled!");
        }
        unsafe { &mut *(self.base_address as *mut HvisorDeviceRegion) }
    }

    pub const fn default() -> Self {
        HvisorDevice {
            base_address: 0,
            is_enable: false,
        }
    }

    pub fn set_base_addr(&mut self, base_addr: usize) {
        self.base_address = base_addr;
        self.is_enable = true;
    }

    pub fn is_full(&self) -> bool {
        self.region().nreq >= MAX_REQ as u32
    }

    pub fn push_req(&mut self, req: HvisorDeviceReq) {
        let region = self.region();
        if region.nreq >= MAX_REQ as u32 {
            panic!("hvisor device region should'nt be full");
        }
        region.req_list[region.nreq as usize] = req;
        region.nreq += 1;
    }

    pub fn get_result(&self) -> &HvisorDeviceRes {
        let res = &self.region().res as *const HvisorDeviceRes;
        unsafe { &*res }
    }
}

/// El1 and EL2 shared region for virtio requests and results.
#[repr(C)]
pub struct HvisorDeviceRegion {
    nreq: u32,
    req_list: [HvisorDeviceReq; MAX_REQ],
    res: HvisorDeviceRes,
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

/// Hvisor device result
#[repr(C)]
pub struct HvisorDeviceRes {
    pub src_cpu: u64,
    /// For notify req, it is the irq_id
    pub value: u64,
    pub is_cfg: u8,
}

impl From<VirtioReq> for HvisorDeviceReq {
    fn from(value: VirtioReq) -> Self {
        Self {
            src_cell: value.src_cell,
            src_cpu: value.src_cpu,
            address: value.mmio.address as u64,
            size: value.mmio.size,
            is_wirte: if value.mmio.is_write { 1 } else { 0 },
            value: value.mmio.value,
            is_cfg: if value.is_cfg { 1 } else { 0 },
        }
    }
}

///  When there are new virtio requests, root cell calls this function.
pub fn handle_virtio_requests() {
    info!("handle virtio requests");
    let mut req_list = VIRTIO_REQ_LIST.lock();
    let mut dev = HVISOR_DEVICE.lock();
    if !dev.is_enable {
        panic!("dev is not enabled");
    }
    while !req_list.is_empty() {
        if dev.is_full() {
            info!("dev is full");
            break;
        }
        let req = req_list.pop_front().unwrap();
        // TODO: 为了避免处理请求时，其他VM需要等待，因此可以尝试拿到一个请求就释放req_list. 不确定会不会提高性能
        let hreq: HvisorDeviceReq = req.into();
        dev.push_req(hreq);
    }
    inject_irq(IRQHVI, false);
    info!("back to el1 from virtio handler");
}
