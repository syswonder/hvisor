use core::sync::atomic::fence;
use core::sync::atomic::Ordering;
use spin::Mutex;

use super::virtio::{VirtioReq, TRAMPOLINE_REQ_LIST};
pub const MAX_REQ: u32 = 4;
pub static HVISOR_DEVICE: Mutex<HvisorDevice> = Mutex::new(HvisorDevice::default());
pub struct HvisorDevice {
    base_address: usize, // el1 and el2 shared region addr, el2 virtual address
    is_enable: bool,
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
        }
    }

    pub fn set_base_addr(&mut self, base_addr: usize) {
        self.base_address = base_addr;
        self.is_enable = true;
    }

    pub fn is_req_list_full(&self) -> bool {
        let region = self.region();
        if ((region.req_rear + 1) & (MAX_REQ - 1)) == region.req_front {
            info!("hvisor req queue full");
            true
        } else {
            false
        }
    }

    pub fn is_res_list_empty(&self) -> bool {
        let region = self.region();
        if region.res_rear == region.res_front {
            true
        } else {
            false
        }
    }

    pub fn push_req(&mut self, req: HvisorDeviceReq) {
        let region = self.region();
        region.req_list[(region.req_rear % MAX_REQ) as usize] = req;
        // Write barrier so that virtio device sees changes to req_list before change to req_idx
        fence(Ordering::SeqCst);
        region.req_rear = (region.req_rear + 1) % MAX_REQ;
        // Write barrier so that device can see change after this method returns
        fence(Ordering::SeqCst);
    }
}

/// El1 and EL2 shared region for virtio requests and results.
#[repr(C)]
pub struct HvisorDeviceRegion {
    /// The first elem of req list, only virtio device updates
    pub req_front: u32,
    /// The last elem's next place of req list, only hvisor updates
    pub req_rear: u32,
    /// The first elem of res list, only hvisor updates
    pub res_front: u32,
    /// The last elem's next place of res list, only virtio device updates
    res_rear: u32,
    pub req_list: [HvisorDeviceReq; MAX_REQ as usize],
    pub res_list: [HvisorDeviceRes; MAX_REQ as usize],
}

/// Hvisor device requests
#[repr(C)]
pub struct HvisorDeviceReq {
    pub src_cpu: u64,
    address: u64,
    size: u64,
    pub value: u64,
    src_cell: u32,
    is_write: u8,
    pub need_interrupt: u8,
}

#[repr(C)]
pub struct HvisorDeviceRes {
    pub target: u64,
    pub value: u64,
    pub res_type: u8, // 0 : no interrupt to cpu ; 1 : interrupt to cpu; 2 : interrupt to a cell
}

impl HvisorDeviceReq {
    pub fn new(
        src_cpu: u64,
        address: u64,
        size: u64,
        value: u64,
        src_cell: u32,
        is_write: bool,
        need_interrupt: u8,
    ) -> Self {
        let is_write = if is_write { 1 } else { 0 };
        Self {
            src_cpu,
            address,
            size,
            value,
            src_cell,
            is_write,
            need_interrupt,
        }
    }
}
impl From<VirtioReq> for HvisorDeviceReq {
    fn from(value: VirtioReq) -> Self {
        Self {
            src_cell: value.src_cell,
            src_cpu: value.src_cpu,
            address: value.mmio.address as u64,
            size: value.mmio.size,
            is_write: if value.mmio.is_write { 1 } else { 0 },
            value: value.mmio.value,
            need_interrupt: if value.is_cfg { 1 } else { 0 },
        }
    }
}
//
// ///  When there are new virtio requests, root cell calls this function.
// pub fn handle_virtio_requests() {
//     debug!("handle virtio requests");
//     let mut dev = HVISOR_DEVICE.lock();
//     assert_eq!(dev.is_enable, true);
//     if dev.is_req_list_full() {
//         // When req list is full, just return.
//         // When root calls finish req hvc, it will call this function again.
//         info!("back to el1 from virtio handler");
//         return;
//     }
//     let mut req_list = TRAMPOLINE_REQ_LIST.lock();
//     while !req_list.is_empty() {
//         if dev.is_req_list_full() {
//             break;
//         }
//         let req = req_list.pop_front().unwrap();
//         let hreq: HvisorDeviceReq = req.into();
//         dev.push_req(hreq);
//     }
//     info!("back to el1 from virtio handler");
// }
