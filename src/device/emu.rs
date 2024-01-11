use spin::Mutex;

use super::virtio::{VirtioReq, TRAMPOLINE_REQ_LIST};
pub const MAX_REQ: u32 = 4;
pub static HVISOR_DEVICE: Mutex<HvisorDevice> = Mutex::new(HvisorDevice::default());
pub struct HvisorDevice {
    base_address: usize, // el1 and el2 shared region addr, el2 virtual address
    is_enable: bool,
    pub shadow_last_req_idx: u32,
}

impl HvisorDevice {
    pub fn region(&self) -> &mut HvisorDeviceRegion {
        if !self.is_enable {
            panic!("hvisor device region is not enabled!");
        }
        unsafe { &mut *(self.base_address as *mut HvisorDeviceRegion) }
    }

    pub const fn default() -> Self {
        HvisorDevice {
            base_address: 0,
            is_enable: false,
            shadow_last_req_idx: 0,
        }
    }

    pub fn set_base_addr(&mut self, base_addr: usize) {
        self.base_address = base_addr;
        self.is_enable = true;
    }

    pub fn is_full(&self) -> bool {
        let region = self.region();
        if region.idx - region.last_req_idx == MAX_REQ as u32 {
            info!("hvisor req queue full");
            true
        } else {
            false
        }
    }

    pub fn push_req(&mut self, req: HvisorDeviceReq) {
        let region = self.region();
        region.req_list[(region.idx % MAX_REQ)as usize] = req;
        region.idx += 1;
    }

}

/// El1 and EL2 shared region for virtio requests and results.
#[repr(C)]
pub struct HvisorDeviceRegion {
    idx: u32,
    pub last_req_idx: u32,
    pub req_list: [HvisorDeviceReq; MAX_REQ as usize],
}

/// Hvisor device requests
#[repr(C)]
pub struct HvisorDeviceReq {
    pub src_cpu: u64,
    address: u64,
    size: u64,
    pub value: u64,
    src_cell: u32,
    is_wirte: u8,
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
    debug!("handle virtio requests");
    let mut dev = HVISOR_DEVICE.lock();
    assert_eq!(dev.is_enable, true);
    if dev.is_full() {
        // When req list is full, just return. 
        // When root calls finish req hvc, it will call this function again. 
        return;
    }
    let mut req_list = TRAMPOLINE_REQ_LIST.lock();
    while !req_list.is_empty() {
        if dev.is_full() {
            break;
        }
        let req = req_list.pop_front().unwrap();
        let hreq: HvisorDeviceReq = req.into();
        dev.push_req(hreq);
    }
    debug!("back to el1 from virtio handler");
}
